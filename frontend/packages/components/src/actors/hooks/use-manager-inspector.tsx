import { createContext, useContext, useEffect, useRef, useState } from "react";
import {
	createWebsocket,
	useWebSocket,
	type UseWebSocketReturn,
} from "./use-websocket";
import { protocol } from "@rivetkit/core/inspector";
import type z from "zod";

type ToClient = z.infer<typeof protocol.manager.ToClientSchema>;
type ToServer = z.infer<typeof protocol.manager.ToServerSchema>;

const ManagerInspectorContext = createContext<
	UseWebSocketReturn<ToServer, ToClient>
	// biome-ignore lint/suspicious/noExplicitAny: <explanation>
>(null as any);

export function ManagerInspectorProvider({
	url,
	children,
}: { children: React.ReactNode; url: string }) {
	const [wsStore] = useState(
		createWebsocket(
			url,
			protocol.manager.ToServerSchema,
			protocol.manager.ToClientSchema,
			{ reconnectDelay: 750, heartbeatInterval: 1000 },
		),
	);

	const managerWsStore = useWebSocket(wsStore);

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		managerWsStore.connect();
		return () => {
			managerWsStore.disconnect();
		};
	}, [url]);

	return (
		<ManagerInspectorContext.Provider value={managerWsStore}>
			{children}
		</ManagerInspectorContext.Provider>
	);
}

export function useManagerInspector() {
	return useContext(ManagerInspectorContext);
}

export function useManagerInspectorListener(cb: (message: ToClient) => void) {
	const listenerRef = useRef(cb);

	const managerInspector = useManagerInspector();

	useEffect(() => {
		listenerRef.current = cb;
	});

	useEffect(() => {
		if (!managerInspector) return;

		const unsubscribe = managerInspector.on((message) => {
			listenerRef.current(message);
		});
		return () => {
			unsubscribe();
		};
	}, [managerInspector]);
}
