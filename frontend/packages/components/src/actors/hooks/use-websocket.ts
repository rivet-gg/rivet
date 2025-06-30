import { useSyncExternalStore } from "react";
import type { z } from "zod";

export interface WebSocketState {
	isConnecting: boolean;
	isConnected: boolean;
	isDisconnected: boolean;
	error: Error | null;
}

export interface WebSocketStore<TClientMessage, TServerMessage> {
	getSnapshot: () => WebSocketState;
	subscribe: (callback: () => void) => () => void;
	subscribeToMessages: (
		callback: (message: TServerMessage) => void,
	) => () => void;
	send: (message: TClientMessage) => void;
	connect: () => void;
	disconnect: () => void;
	destroy: () => void;
}

export function createWebsocket<TClientMessage, TServerMessage>(
	url: string,
	clientMessageSchema: z.ZodSchema<TClientMessage>,
	serverMessageSchema: z.ZodSchema<TServerMessage>,
	options: {
		autoReconnect?: boolean;
		reconnectDelay?: number;
		connectionTimeout?: number;
		heartbeatInterval?: number;
	} = {},
): WebSocketStore<TClientMessage, TServerMessage> {
	const {
		autoReconnect = true,
		reconnectDelay = 3000,
		connectionTimeout: connectionTimeoutMs = 10000,
		heartbeatInterval = 30000,
	} = options;
	let websocket: WebSocket | null = null;
	let reconnectTimeout: NodeJS.Timeout | null = null;
	let connectionTimeoutId: NodeJS.Timeout | null = null;
	let heartbeatTimeoutId: NodeJS.Timeout | null = null;
	let shouldReconnect = false;

	const subscribers = new Set<() => void>();
	const messageSubscribers = new Set<(message: TServerMessage) => void>();

	let state: WebSocketState = {
		isConnecting: false,
		isConnected: false,
		isDisconnected: true,
		error: null,
	};

	const updateState = (newState: Partial<WebSocketState>) => {
		state = { ...state, ...newState };
		for (const callback of subscribers) {
			callback();
		}
	};

	const clearAllTimeouts = () => {
		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
			reconnectTimeout = null;
		}
		if (connectionTimeoutId) {
			clearTimeout(connectionTimeoutId);
			connectionTimeoutId = null;
		}
		if (heartbeatTimeoutId) {
			clearTimeout(heartbeatTimeoutId);
			heartbeatTimeoutId = null;
		}
	};

	const startHeartbeat = () => {
		if (heartbeatTimeoutId) {
			clearTimeout(heartbeatTimeoutId);
		}
		heartbeatTimeoutId = setTimeout(() => {
			if (websocket?.readyState === WebSocket.OPEN) {
				// Restart heartbeat if still connected
				startHeartbeat();
			} else if (shouldReconnect && autoReconnect) {
				// Connection seems dead, trigger reconnection
				websocket?.close();
			}
		}, heartbeatInterval);
	};

	const connect = () => {
		if (
			websocket?.readyState === WebSocket.CONNECTING ||
			websocket?.readyState === WebSocket.OPEN
		) {
			return;
		}

		// Clear any existing timeouts
		clearAllTimeouts();

		shouldReconnect = true;

		updateState({
			isConnecting: true,
			isConnected: false,
			isDisconnected: false,
			error: null,
		});

		// Set connection timeout
		connectionTimeoutId = setTimeout(() => {
			if (websocket?.readyState === WebSocket.CONNECTING) {
				const timeoutError = new Error(
					`Connection timeout after ${connectionTimeoutMs}ms`,
				);
				websocket.close();
				updateState({
					isConnecting: false,
					isConnected: false,
					isDisconnected: true,
					error: timeoutError,
				});

				// Attempt to reconnect on timeout if autoReconnect is enabled
				if (shouldReconnect && autoReconnect) {
					reconnectTimeout = setTimeout(() => {
						if (shouldReconnect) {
							connect();
						}
					}, reconnectDelay);
				}
			}
		}, connectionTimeoutMs);

		try {
			const urlWithProtocol =
				url.startsWith("ws://") || url.startsWith("wss://")
					? url
					: `ws://${url}`;
			const fullUrl = `${urlWithProtocol}/registry/inspect`;
			websocket = new WebSocket(fullUrl);

			websocket.onopen = () => {
				// Clear connection timeout on successful connection
				if (connectionTimeoutId) {
					clearTimeout(connectionTimeoutId);
					connectionTimeoutId = null;
				}

				updateState({
					isConnecting: false,
					isConnected: true,
					isDisconnected: false,
					error: null,
				});

				// Start heartbeat monitoring
				startHeartbeat();
			};

			websocket.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					const validatedMessage = serverMessageSchema.parse(data);
					// Notify message subscribers
					for (const callback of messageSubscribers) {
						callback(validatedMessage);
					}
				} catch (error) {
					console.error("Failed to parse WebSocket message:", error);
					updateState({
						error:
							error instanceof Error
								? error
								: new Error("Failed to parse message"),
					});
				}
			};

			websocket.onclose = (event) => {
				// Clear all timeouts when connection closes
				clearAllTimeouts();

				updateState({
					isConnecting: false,
					isConnected: false,
					isDisconnected: true,
				});

				// Only attempt to reconnect if it wasn't a manual disconnect and autoReconnect is enabled
				if (shouldReconnect && autoReconnect && !event.wasClean) {
					reconnectTimeout = setTimeout(() => {
						if (shouldReconnect) {
							connect();
						}
					}, reconnectDelay);
				}
			};

			websocket.onerror = () => {
				const error = new Error("WebSocket connection error");
				updateState({
					isConnecting: false,
					isConnected: false,
					isDisconnected: true,
					error,
				});
			};
		} catch (error) {
			updateState({
				isConnecting: false,
				isConnected: false,
				isDisconnected: true,
				error:
					error instanceof Error
						? error
						: new Error("Failed to create WebSocket"),
			});

			// Attempt to reconnect on connection error if autoReconnect is enabled
			if (shouldReconnect && autoReconnect) {
				reconnectTimeout = setTimeout(() => {
					if (shouldReconnect) {
						connect();
					}
				}, reconnectDelay);
			}
		}
	};

	const disconnect = () => {
		shouldReconnect = false;

		// Clear all timeouts
		clearAllTimeouts();

		if (websocket) {
			websocket.close();
			websocket = null;
		}

		updateState({
			isConnecting: false,
			isConnected: false,
			isDisconnected: true,
		});
	};

	const send = (message: TClientMessage) => {
		if (!websocket || websocket.readyState !== WebSocket.OPEN) {
			throw new Error("WebSocket is not connected");
		}

		try {
			// Validate the message before sending
			const validatedMessage = clientMessageSchema.parse(message);
			websocket.send(JSON.stringify(validatedMessage));
		} catch (error) {
			const sendError =
				error instanceof Error
					? error
					: new Error("Failed to send message");
			updateState({ error: sendError });
			throw sendError;
		}
	};

	const getSnapshot = () => state;

	const subscribe = (callback: () => void) => {
		subscribers.add(callback);
		return () => {
			subscribers.delete(callback);
		};
	};

	const subscribeToMessages = (
		callback: (message: TServerMessage) => void,
	) => {
		messageSubscribers.add(callback);
		return () => {
			messageSubscribers.delete(callback);
		};
	};

	const destroy = () => {
		disconnect();
		subscribers.clear();
		messageSubscribers.clear();
	};

	return {
		getSnapshot,
		subscribe,
		subscribeToMessages,
		send,
		connect,
		disconnect,
		destroy,
	};
}

export type UseWebSocketReturn<TClientMessage, TServerMessage> =
	WebSocketState & {
		send: (message: TClientMessage) => void;
		on: (callback: (message: TServerMessage) => void) => () => void;
		connect: () => void;
		disconnect: () => void;
	};

export function useWebSocket<TClientMessage, TServerMessage>(
	store: WebSocketStore<TClientMessage, TServerMessage>,
) {
	const state = useSyncExternalStore(store.subscribe, store.getSnapshot);

	return {
		...state,
		on: store.subscribeToMessages,
		send: store.send,
		connect: store.connect,
		disconnect: store.disconnect,
	};
}
