import {
	HEADER_CONN_PARAMS,
	HEADER_ENCODING,
} from "@/common/actor-router-consts";
import type * as protocol from "@/common/client-protocol";
import {
	CURRENT_VERSION as CLIENT_PROTOCOL_CURRENT_VERSION,
	HTTP_QUEUE_SEND_REQUEST_VERSIONED,
	HTTP_QUEUE_SEND_RESPONSE_VERSIONED,
} from "@/common/client-protocol-versioned";
import {
	type HttpQueueSendRequest as HttpQueueSendRequestJson,
	HttpQueueSendRequestSchema,
	type HttpQueueSendResponse as HttpQueueSendResponseJson,
	HttpQueueSendResponseSchema,
} from "@/common/client-protocol-zod";
import type { Encoding, JsonCompatValue } from "@/common/encoding";
import { decodeCborCompat, encodeCborCompat } from "@/serde";
import { bufferToArrayBuffer } from "@/utils";
import { sendHttpRequest } from "./utils";

/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
export interface QueueSender {
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	send(
		name: string,
		body: unknown,
		options: QueueSendWaitOptions,
	): Promise<QueueSendResult>;
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	send(
		name: string,
		body: unknown,
		options?: QueueSendNoWaitOptions,
	): Promise<void>;
}

/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
export interface QueueSendWaitOptions {
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	wait: true;
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	timeout?: number;
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	signal?: AbortSignal;
}

/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
export interface QueueSendNoWaitOptions {
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	wait?: false;
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	timeout?: never;
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	signal?: AbortSignal;
}

/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
export type QueueSendOptions = QueueSendWaitOptions | QueueSendNoWaitOptions;

/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
export interface QueueSendResult<TResponse = unknown> {
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	status: "completed" | "timedOut";
	/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
	response?: TResponse;
}

interface QueueSenderOptions {
	encoding: Encoding;
	params: unknown;
	/** Ray ID and trace context headers, read per send. */
	telemetryHeaders: () => Record<string, string>;
	customFetch: (request: Request) => Promise<Response>;
}
/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
export function createQueueSender(
	senderOptions: QueueSenderOptions,
): QueueSender {
	async function send(
		name: string,
		body: unknown,
		options: QueueSendWaitOptions,
	): Promise<QueueSendResult>;
	async function send(
		name: string,
		body: unknown,
		options?: QueueSendNoWaitOptions,
	): Promise<void>;
	async function send(
		name: string,
		body: unknown,
		options?: QueueSendOptions,
	): Promise<QueueSendResult | void> {
		const wait = options?.wait ?? false;
		const timeout = options?.timeout;

		const result = await sendHttpRequest<
			protocol.HttpQueueSendRequest,
			protocol.HttpQueueSendResponse,
			HttpQueueSendRequestJson,
			HttpQueueSendResponseJson,
			{
				/** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */
				body: unknown /** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */;
				wait?: boolean /** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */;
				timeout?: number /** @deprecated Queues are deprecated. Use actor streams. See https://rivet.dev/actors/docs/migrate-queues-to-streams. */;
				name?: string;
			},
			QueueSendResult
		>({
			url: `http://actor/queue/${encodeURIComponent(name)}`,
			method: "POST",
			headers: {
				[HEADER_ENCODING]: senderOptions.encoding,
				...senderOptions.telemetryHeaders(),
				...(senderOptions.params !== undefined
					? {
							[HEADER_CONN_PARAMS]: JSON.stringify(
								senderOptions.params,
							),
						}
					: {}),
			},
			body: { body, wait, timeout },
			encoding: senderOptions.encoding,
			customFetch: senderOptions.customFetch,
			signal: options?.signal,
			requestVersion: CLIENT_PROTOCOL_CURRENT_VERSION,
			requestVersionedDataHandler: HTTP_QUEUE_SEND_REQUEST_VERSIONED,
			responseVersion: CLIENT_PROTOCOL_CURRENT_VERSION,
			responseVersionedDataHandler: HTTP_QUEUE_SEND_RESPONSE_VERSIONED,
			requestZodSchema: HttpQueueSendRequestSchema,
			responseZodSchema: HttpQueueSendResponseSchema,
			requestToJson: (value): HttpQueueSendRequestJson => ({
				...value,
				name,
			}),
			requestToBare: (value): protocol.HttpQueueSendRequest => ({
				name: value.name ?? name,
				body: bufferToArrayBuffer(
					encodeCborCompat(value.body as JsonCompatValue),
				),
				wait: value.wait ?? false,
				timeout:
					value.timeout !== undefined ? BigInt(value.timeout) : null,
			}),
			responseFromJson: (json): QueueSendResult => {
				if (json.response === undefined) {
					return { status: json.status as "completed" | "timedOut" };
				}
				return {
					status: json.status as "completed" | "timedOut",
					response: json.response,
				};
			},
			responseFromBare: (bare): QueueSendResult => {
				if (bare.response === null || bare.response === undefined) {
					return { status: bare.status as "completed" | "timedOut" };
				}
				return {
					status: bare.status as "completed" | "timedOut",
					response: decodeCborCompat(new Uint8Array(bare.response)),
				};
			},
		});

		if (wait) {
			return result;
		}
		return;
	}

	return {
		send,
	};
}
