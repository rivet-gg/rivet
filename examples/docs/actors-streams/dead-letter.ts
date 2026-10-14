import { actor } from "rivetkit";

export const worker = actor({
	stream: {
		deadLetter: true,
		async onMessage(_c, { body }: { body: { url: string } }) {
			const response = await fetch(body.url);
			if (!response.ok)
				throw new Error(`Request failed: ${response.status}`);
		},
		onError(_c, failure) {
			console.error("Retries exhausted", failure.stream, failure.error);
		},
	},
});
