import type { DeadLetterMessage, StreamWriter } from "rivetkit";

export async function reprocess(
	stream: StreamWriter<{ url: string }>,
	message: DeadLetterMessage<{ url: string }>,
) {
	// Reuse this key if the request times out. Use a new key for a new replay.
	return stream.redrive(message, {
		idempotencyKey: `repair-1:${message.id}`,
	});
}
