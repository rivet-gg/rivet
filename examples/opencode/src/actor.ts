import { createHash } from "node:crypto";
import { resolve } from "node:path";
import type { OpenCode } from "@opencode/sdk/effect";
import { Effect, Scope, Stream } from "effect";
import { type ActorContext, actor, event } from "rivetkit";
import { type DatabaseProvider, db, type RawAccess } from "rivetkit/db";
import { createHost, type OpenCodeEvent, type OpenCodeHost } from "./host.js";

type Context = ActorContext<
	any,
	any,
	any,
	any,
	any,
	DatabaseProvider<RawAccess>,
	any,
	any
>;
type SessionID = Parameters<
	OpenCode.Interface["session"]["get"]
>[0]["sessionID"];
type Directory = NonNullable<
	NonNullable<
		Parameters<OpenCode.Interface["session"]["create"]>[0]
	>["location"]
>["directory"];
type Session = Effect.Success<
	ReturnType<OpenCode.Interface["session"]["create"]>
>;

const hosts = new Map<string, Promise<OpenCodeHost>>();
const stopping = new Set<string>();
const active = new Map<string, Map<string, Promise<void>>>();
const sessions = new Map<string, Promise<Session>>();
const cwd = resolve(process.env.OPENCODE_WORKDIR ?? process.cwd());

function sessionID(c: Context): SessionID {
	return `ses_${createHash("sha256").update(c.actorId).digest("hex").slice(0, 26)}` as SessionID;
}

function keepRunning(c: Context, host: OpenCodeHost, id: SessionID) {
	let waits = active.get(c.actorId);
	if (!waits) {
		waits = new Map();
		active.set(c.actorId, waits);
	}
	if (waits.has(id) || stopping.has(c.actorId)) return;
	const pending = c
		.keepAwake(
			Effect.runPromise(host.client.session.wait({ sessionID: id })),
		)
		.catch((error) => {
			if (!stopping.has(c.actorId))
				c.log.error({
					msg: "OpenCode session failed",
					error: String(error),
				});
		})
		.finally(() => waits?.delete(id));
	waits.set(id, pending);
}

function getHost(c: Context): Promise<OpenCodeHost> {
	if (stopping.has(c.actorId))
		return Promise.reject(new Error("OpenCode actor is stopping"));
	const existing = hosts.get(c.actorId);
	if (existing) return existing;
	const ready = (async () => {
		const host = await createHost(c.db);
		try {
			await Effect.runPromise(
				host.client.event.subscribe().pipe(
					Stream.runForEach((value) =>
						Effect.sync(() => {
							if (value.type === "session.execution.started")
								keepRunning(c, host, value.data.sessionID);
							c.broadcast("event", value);
						}),
					),
					Effect.forkScoped,
					Scope.provide(host.scope),
				),
			);
			await host.resume();
			for (const id of Object.keys(
				await Effect.runPromise(host.client.session.active()),
			))
				keepRunning(c, host, id as SessionID);
			return host;
		} catch (error) {
			await host.close();
			throw error;
		}
	})();
	hosts.set(c.actorId, ready);
	void ready.catch(() => {
		if (hosts.get(c.actorId) === ready) hosts.delete(c.actorId);
	});
	return ready;
}

function getSession(c: Context, host: OpenCodeHost): Promise<Session> {
	const existing = sessions.get(c.actorId);
	if (existing) return existing;
	const id = sessionID(c);
	const pending = Effect.runPromise(
		host.client.session.get({ sessionID: id }).pipe(
			Effect.catchTag("SessionNotFoundError", () =>
				host.client.session.create({
					id,
					location: { directory: cwd as Directory },
				}),
			),
		),
	);
	sessions.set(c.actorId, pending);
	void pending.catch(() => {
		if (sessions.get(c.actorId) === pending) sessions.delete(c.actorId);
	});
	return pending;
}

async function closeHost(c: Context) {
	stopping.add(c.actorId);
	try {
		await (await hosts.get(c.actorId)?.catch(() => undefined))?.close();
	} finally {
		hosts.delete(c.actorId);
		active.delete(c.actorId);
		sessions.delete(c.actorId);
		stopping.delete(c.actorId);
	}
}

export const opencode = actor({
	options: { actionTimeout: 2_147_483_647, sleepGracePeriod: 15 * 60_000 },
	db: db(),
	events: { event: event<OpenCodeEvent>() },
	onWake: async (c) => {
		await getHost(c);
	},
	onSleep: closeHost,
	onDestroy: closeHost,
	actions: {
		getSession: async (c) => {
			const host = await getHost(c);
			return c.keepAwake(getSession(c, host));
		},
		prompt: async (c, text: string) => {
			const host = await getHost(c);
			return c.keepAwake(
				(async () => {
					const session = await getSession(c, host);
					const result = await Effect.runPromise(
						host.client.session.prompt({
							sessionID: session.id,
							text,
						}),
					);
					keepRunning(c, host, session.id);
					return result;
				})(),
			);
		},
		getMessages: async (c) => {
			const host = await getHost(c);
			return c.keepAwake(
				(async () =>
					Effect.runPromise(
						host.client.message.list({
							sessionID: (await getSession(c, host)).id,
						}),
					))(),
			);
		},
		waitForIdle: async (c) => {
			const host = await getHost(c);
			return c.keepAwake(
				(async () =>
					Effect.runPromise(
						host.client.session.wait({
							sessionID: (await getSession(c, host)).id,
						}),
					))(),
			);
		},
		abort: async (c) => {
			const host = await getHost(c);
			return c.keepAwake(
				(async () =>
					Effect.runPromise(
						host.client.session.interrupt({
							sessionID: (await getSession(c, host)).id,
						}),
					))(),
			);
		},
	},
});
