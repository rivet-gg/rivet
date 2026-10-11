import { actor, setup } from "rivetkit";
import { z } from "zod";

export const customer = actor({
	state: { total: 0 },
	stream: {
		onMessage(c, { body }: { body: { amount: number } }) {
			c.state.total += body.amount;
		},
	},
	subscriptions: {
		orders: {
			stream: "/tenants/:tenant/orders/:customer",
			actorKey: [":tenant", ":customer"],
			schema: z.object({ amount: z.number() }),
			onMessage(c, { body }) {
				c.state.total += body.amount;
			},
		},
	},
});

export const registry = setup({
	use: { customer },
	durableStreams: {
		endpoint: process.env.DURABLE_STREAMS_ENDPOINT!,
	},
});
