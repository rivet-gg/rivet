import { z } from "zod/v4";
import { actor, setup } from "../../src/mod";
import { buildNativeRegistry } from "../../src/registry/native";

const worker = actor({
	state: { messages: [] as number[] },
	options: { sleepTimeout: 500 },
	stream: {
		async onMessage(
			c,
			message: { body: { id: number; delay?: number } },
			{ signal },
		) {
			if (message.body.delay)
				await new Promise<void>((resolve, reject) => {
					const timer = setTimeout(resolve, message.body.delay);
					signal.addEventListener(
						"abort",
						() => {
							clearTimeout(timer);
							reject(signal.reason);
						},
						{ once: true },
					);
				});
			c.state.messages.push(message.body.id);
		},
	},
	actions: {
		snapshot: (c) => [...c.state.messages],
		sleep: async (c) => {
			await c.sleep();
		},
		appendSelf: async (c, id: number) => {
			await c.stream.append({ id });
		},
	},
});

const validated = actor({
	state: { messages: [] as number[] },
	options: { sleepTimeout: 500 },
	stream: {
		schema: z.object({ id: z.string().transform(Number) }),
		onMessage(c, { body }) {
			c.state.messages.push(body.id);
		},
	},
	actions: { snapshot: (c) => [...c.state.messages] },
});

const recovering = actor({
	state: {
		messages: [] as number[],
		blocked: true,
		failed: false,
		errors: 0,
	},
	options: { sleepTimeout: 500 },
	stream: {
		onError(c) {
			c.state.errors += 1;
		},
		onMessage(c, { body }: { body: { id: number } }) {
			if (body.id === 8 && c.state.blocked) {
				c.state.failed = true;
				throw new Error("Intentional test failure");
			}
			c.state.messages.push(body.id);
		},
	},
	actions: {
		snapshot: (c) => ({
			messages: [...c.state.messages],
			failed: c.state.failed,
			errors: c.state.errors,
		}),
		unblock: (c) => {
			c.state.blocked = false;
		},
	},
});

const deadLetters = actor({
	state: { messages: [] as number[], blocked: true, errors: 0 },
	options: { sleepTimeout: 500 },
	stream: {
		deadLetter: true,
		onError(c) {
			c.state.errors += 1;
		},
		onMessage(c, { body }: { body: { id: number } }) {
			if (body.id === 8 && c.state.blocked)
				throw new Error("Dead-letter test failure");
			c.state.messages.push(body.id);
		},
	},
	actions: {
		snapshot: (c) => ({
			messages: [...c.state.messages],
			errors: c.state.errors,
		}),
		unblock: (c) => {
			c.state.blocked = false;
		},
	},
});

const version = Number(process.env.RIVET_STREAMS_TEST_VERSION ?? "1");
const customer = actor({
	state: { messages: [] as string[] },
	options: { sleepTimeout: 500 },
	stream: {
		onMessage(c, { body }: { body: { id: string } }) {
			c.state.messages.push(`primary:${body.id}`);
		},
	},
	subscriptions: {
		orders: {
			stream: "/tenants/:tenant/orders/:customer/:source",
			actorKey: [":tenant", ":customer"],
			schema: z.object({ id: z.string() }),
			onMessage(c, { body }) {
				if (body.id === "poison") throw new Error("Poison order");
				c.state.messages.push(`orders:${body.id}`);
			},
		},
		notifications: {
			stream: "/tenants/:tenant/notifications/:customer",
			actorKey: [":tenant", ":customer"],
			onMessage(c, { body }: { body: { id: string } }) {
				c.state.messages.push(`notifications:${body.id}`);
			},
		},
	},
	actions: { snapshot: (c) => [...c.state.messages] },
});
// Deploy the same fixture with successive complete declaration sets.
if (version < 2) delete customer.config.subscriptions!.notifications;
if (version >= 3) delete customer.config.subscriptions!.orders;
const actors = { worker, validated, recovering, customer, deadLetters };
if (version >= 4) Reflect.deleteProperty(actors, "customer");

const registry = setup({
	use: actors,
	endpoint: process.env.RIVET_ENDPOINT!,
	token: process.env.RIVET_TOKEN!,
	namespace: "default",
	startServices: false,
	sqlite: "remote",
	envoy: { poolName: process.env.RIVET_STREAMS_TEST_POOL!, version },
	durableStreams: {
		endpoint: process.env.RIVET_STREAMS_TEST_ROOT!,
	},
});
export type Registry = typeof registry;

const {
	runtime,
	registry: nativeRegistry,
	serveConfig,
} = await buildNativeRegistry(registry.parseConfig());
const shutdown = () => {
	void runtime.shutdownRegistry(nativeRegistry);
};
process.once("SIGTERM", shutdown);
process.once("SIGINT", shutdown);
await runtime.serveRegistry(nativeRegistry, serveConfig);
