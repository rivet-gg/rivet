import type { z } from "zod/v4";

export interface StreamMessage<T> {
	body: T;
}

export interface StreamSource {
	/** Defaults to the actor's primary input. */
	subscription?: string;
	/** Relative DS path; required for named subscriptions. */
	stream?: string;
}

export interface StreamFailure {
	subscription: string;
	stream: string;
	offset: string;
	messageIndex: number;
	attempts: number;
	firstFailedAt: number;
	lastFailedAt: number;
	nextAttemptAt: number | null;
	paused: boolean;
	error: { group: string; code: string; message: string };
}

export interface DeadLetterMessage<T = unknown> {
	id: string;
	actor: { id: string; name: string; key: unknown[] };
	subscription: string;
	source: { stream: string; offset: string; messageIndex: number };
	body: T;
	failure: StreamFailure;
}
export interface StreamRedriveReceipt {
	failureId: string;
	idempotencyKey: string;
	stream: string;
	nextOffset: string;
	createdAt: number;
}
export interface StreamStatus {
	deadLetterStream: string | null;
	deadLetter: { stream: string; pending: true; error: string | null } | null;
	subscription: string;
	stream: string;
	offset: string | null;
	completed: number;
	failure: StreamFailure | null;
}

export interface StreamWriter<T> {
	/** Append failed primary input at the tail; reuse the key when retrying this operation. */
	redrive(
		message: DeadLetterMessage<T>,
		options: { idempotencyKey: string },
	): Promise<StreamRedriveReceipt>;
	/** Inspect durable progress and failure state for one source. */
	status(source?: StreamSource): Promise<StreamStatus>;
	/** Retry the paused record with a fresh retry budget. */
	resume(source?: StreamSource): Promise<void>;
	/** Resolves after durable acceptance, before processing completes. */
	append(body: T): Promise<void>;
}

export type StreamInput<S, T> = S extends z.ZodType ? z.input<S> : T;
export type StreamOutput<S, T> = S extends z.ZodType ? z.output<S> : T;

export interface ActorStreamDefinition<C, S extends z.ZodType | undefined, T> {
	schema?: S;
	/** After retries, publish to a separate stream before continuing. */
	deadLetter?: boolean | { stream: string };
	/** Called after retries are exhausted. Without a DLQ, the source remains paused until resumed. */
	// Keep this callback from competing with onMessage when inferring input.
	onError?: NoInfer<(c: C, failure: StreamFailure) => void | Promise<void>>;
	onMessage(
		c: C,
		message: StreamMessage<StreamOutput<S, T>>,
		options: { signal: AbortSignal },
	): void | Promise<void>;
}

export type ActorStreamInput<A> = A extends { readonly "~streamInput": infer T }
	? T
	: never;
