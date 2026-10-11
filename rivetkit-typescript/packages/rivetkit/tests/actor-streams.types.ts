import { expectTypeOf } from "vitest";
import { z } from "zod/v4";
import { actor } from "../src/actor/definition";
import type { ActorStreamInput } from "../src/actor/stream";
import type { ActorHandle } from "../src/client/actor-handle";

const typed = actor({
	state: { count: 0 },
	stream: {
		onError(c, failure) {
			expectTypeOf(c.state.count).toEqualTypeOf<number>();
			expectTypeOf(failure.attempts).toEqualTypeOf<number>();
		},
		async onMessage(c, message: { body: { amount: number } }, { signal }) {
			expectTypeOf(c.state.count).toEqualTypeOf<number>();
			expectTypeOf(signal).toEqualTypeOf<AbortSignal>();
			expectTypeOf(c.stream.append)
				.parameter(0)
				.toEqualTypeOf<{ amount: number }>();
			c.state.count += message.body.amount;
		},
	},
	actions: { count: (c) => c.state.count },
});
expectTypeOf<ActorStreamInput<typeof typed>>().toEqualTypeOf<{
	amount: number;
}>();
declare const typedHandle: ActorHandle<typeof typed>;
typedHandle.stream.append({ amount: 1 });
// @ts-expect-error body inferred from the handler
typedHandle.stream.append({ amount: "invalid" });
expectTypeOf(typedHandle.count()).toEqualTypeOf<Promise<number>>();

const validated = actor({
	state: { count: 0 },
	stream: {
		schema: z.object({ amount: z.string().transform(Number) }),
		async onMessage(c, { body }) {
			expectTypeOf(body.amount).toEqualTypeOf<number>();
			expectTypeOf(c.stream.append)
				.parameter(0)
				.toEqualTypeOf<{ amount: string }>();
			c.state.count += body.amount;
		},
	},
});
expectTypeOf<ActorStreamInput<typeof validated>>().toEqualTypeOf<{
	amount: string;
}>();
declare const validatedHandle: ActorHandle<typeof validated>;
validatedHandle.stream.append({ amount: "1" });
// @ts-expect-error append accepts schema input, not transformed output
validatedHandle.stream.append({ amount: 1 });

actor({
	stream: {
		onMessage(_c, { body }) {
			expectTypeOf(body).toEqualTypeOf<unknown>();
		},
	},
});

actor({
	state: { count: 0 },
	subscriptions: {
		orders: {
			stream: "/tenants/:tenant/orders/:customer",
			actorKey: [":tenant", ":customer"],
			schema: z.object({ amount: z.string().transform(Number) }),
			onMessage(c, { body }, { signal }) {
				expectTypeOf(c.state.count).toEqualTypeOf<number>();
				expectTypeOf(body.amount).toEqualTypeOf<number>();
				expectTypeOf(signal).toEqualTypeOf<AbortSignal>();
				c.state.count += body.amount;
			},
		},
		raw: {
			stream: "/raw/:customer",
			actorKey: [":customer"],
			onMessage(c, message: { body: { amount: number } }) {
				c.state.count += message.body.amount;
			},
		},
	},
});

expectTypeOf(typedHandle.stream.status()).toEqualTypeOf<
	Promise<import("../src/actor/stream").StreamStatus>
>();
expectTypeOf(typedHandle.stream.resume()).toEqualTypeOf<Promise<void>>();

const combined = actor({
    state: { count: 0 },
    stream: {
        onMessage(c, { body }: { body: { amount: number } }) {
            c.state.count += body.amount;
        },
    },
    subscriptions: {
        orders: {
            stream: "/orders/:customer",
            actorKey: [":customer"],
            schema: z.object({ amount: z.number() }),
            onError(c, failure) {
                expectTypeOf(c.state.count).toEqualTypeOf<number>();
                expectTypeOf(failure.stream).toEqualTypeOf<string>();
            },
            onMessage(c, { body }) {
                expectTypeOf(body.amount).toEqualTypeOf<number>();
                c.state.count += body.amount;
            },
        },
    },
});
expectTypeOf<ActorStreamInput<typeof combined>>().toEqualTypeOf<{ amount: number }>();
declare const combinedHandle: ActorHandle<typeof combined>;
combinedHandle.stream.append({ amount: 1 });
// @ts-expect-error Combining primary and named handlers preserves the primary input.
combinedHandle.stream.append({ amount: "wrong" });
