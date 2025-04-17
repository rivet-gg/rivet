export async function safeAsync<T>(
	fn: Promise<T>,
): Promise<[T, undefined] | [undefined, unknown]> {
	try {
		return [await fn, undefined];
	} catch (e) {
		return [undefined, e];
	}
}

// biome-ignore lint/suspicious/noExplicitAny: we need to use any here
export function safe<T, Args extends any[]>(
	fn: (...args: Args) => Promise<T> | T,
): (...args: Args) => Promise<[T, undefined] | [undefined, unknown]> {
	return async (...args: Args) => {
		try {
			return [await fn(...args), undefined];
		} catch (e) {
			return [undefined, e];
		}
	};
}
