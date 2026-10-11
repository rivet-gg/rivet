import { actor, setup } from "rivetkit";

type Input =
	| { type: "credit"; amount: number }
	| { type: "debit"; amount: number };

const account = actor({
	state: { balance: 0 },
	stream: {
		// A schema is optional. The input type is inferred from onMessage.
		onMessage(c, { body }: { body: Input }) {
			if (body.type === "credit") c.state.balance += body.amount;
			else c.state.balance -= body.amount;
		},
	},
	actions: { balance: (c) => c.state.balance },
});

export const registry = setup({ use: { account } });
export type Registry = typeof registry;
