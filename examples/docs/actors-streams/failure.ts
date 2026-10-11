import { actor } from "rivetkit";

export const worker = actor({
	state: { enabled: true, processed: 0 },
	stream: {
		onMessage(c, { body }: { body: { amount: number } }) {
			if (!c.state.enabled) throw new Error("Processing is disabled");
			c.state.processed += body.amount;
		},
		onError(_c, failure) {
			console.error("Stream paused", failure.stream, failure.error);
		},
	},
	actions: {
		enable(c) {
			c.state.enabled = true;
		},
	},
});
