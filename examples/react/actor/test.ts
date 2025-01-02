export function inOutStream() {
	const buffer: Uint8Array[] = [];
	let writer: WritableStreamDefaultWriter<Uint8Array> | null = null;

	const writable = new WritableStream<Uint8Array>({
		write(chunk) {
			buffer.push(chunk);
		},
		close() {},
	});

	const readable = new ReadableStream<Uint8Array>({
		start() {},
		async pull(controller) {
			if (buffer.length > 0) {
				const chunk = buffer.shift(); // Get the next chunk from the buffer
				if (chunk) {
					controller.enqueue(chunk); // Push it to the readable stream
				}
			} else {
				if (writable.locked) {
					await new Promise((resolve) => setTimeout(resolve, 10));
					return this.pull?.(controller);
				}
				return controller.close();
			}
		},
		cancel() {},
	});

	return {
		readable,
		on: (str: string, fn: () => void) => {
			if (str === "drain") {
				writer = writable.getWriter();
				fn();
			}
		},
		write(chunk: Uint8Array) {
			writer?.write(chunk);
		},
		flush() {
			writer?.close();
		},
		end() {
			writer?.releaseLock();
		},
	};
}
