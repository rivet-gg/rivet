import { createHash } from "node:crypto";
import { resolve } from "node:path";
import type { OpenCode } from "@opencode/sdk/effect";
import { Effect, Scope, Stream } from "effect";
import type { ActorContext } from "rivetkit";
import type { DatabaseProvider, RawAccess } from "rivetkit/db";
import { createHost } from "./host.js";

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

const cwd = resolve(process.env.OPENCODE_WORKDIR ?? process.cwd());

// Called once per actor wake. Rivet holds the returned promise in actor-local vars,
// like the retained host promise in OpenCode's Cloudflare Durable Object example.
export async function createRuntime(c: Context) {
	const host = await createHost(c.db);
	const id =
		`ses_${createHash("sha256").update(c.actorId).digest("hex").slice(0, 26)}` as SessionID;
	const active = new Map<string, Promise<void>>();
	let session: Promise<Session> | undefined;
	let closing = false;

	function keepRunning(sessionID: SessionID) {
		if (active.has(sessionID) || closing) return;
		const pending = c
			.keepAwake(
				Effect.runPromise(host.client.session.wait({ sessionID })),
			)
			.catch((error) => {
				if (!closing)
					c.log.error({
						msg: "OpenCode session failed",
						error: String(error),
					});
			})
			.finally(() => active.delete(sessionID));
		active.set(sessionID, pending);
	}

	function getSession(): Promise<Session> {
		if (session !== undefined) return session;
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
		session = pending;
		void pending.catch(() => {
			if (session === pending) session = undefined;
		});
		return pending;
	}

	try {
		await Effect.runPromise(
			host.client.event.subscribe().pipe(
				Stream.runForEach((value) =>
					Effect.sync(() => {
						if (value.type === "session.execution.started")
							keepRunning(value.data.sessionID);
						c.broadcast("event", value);
					}),
				),
				Effect.forkScoped,
				Scope.provide(host.scope),
			),
		);
		await host.resume();
		for (const sessionID of Object.keys(
			await Effect.runPromise(host.client.session.active()),
		))
			keepRunning(sessionID as SessionID);
	} catch (error) {
		await host.close();
		throw error;
	}

	return {
		close: async () => {
			closing = true;
			await host.close();
		},
		getSession: (ctx: Context) => ctx.keepAwake(getSession()),
		prompt: (ctx: Context, text: string) =>
			ctx.keepAwake(
				(async () => {
					const current = await getSession();
					const result = await Effect.runPromise(
						host.client.session.prompt({
							sessionID: current.id,
							text,
						}),
					);
					keepRunning(current.id);
					return result;
				})(),
			),
		getMessages: (ctx: Context) =>
			ctx.keepAwake(
				(async () =>
					Effect.runPromise(
						host.client.message.list({
							sessionID: (await getSession()).id,
						}),
					))(),
			),
		waitForIdle: (ctx: Context) =>
			ctx.keepAwake(
				(async () =>
					Effect.runPromise(
						host.client.session.wait({
							sessionID: (await getSession()).id,
						}),
					))(),
			),
		abort: (ctx: Context) =>
			ctx.keepAwake(
				(async () =>
					Effect.runPromise(
						host.client.session.interrupt({
							sessionID: (await getSession()).id,
						}),
					))(),
			),
	};
}

export type OpenCodeRuntime = Awaited<ReturnType<typeof createRuntime>>;

export function requireRuntime(
	pending: Promise<OpenCodeRuntime> | undefined,
): Promise<OpenCodeRuntime> {
	if (pending === undefined) throw new Error("OpenCode host has not started");
	return pending;
}

export async function closeRuntime(c: {
	vars: { opencode: Promise<OpenCodeRuntime> | undefined };
}) {
	const pending = c.vars.opencode;
	c.vars.opencode = undefined;
	await (await pending?.catch(() => undefined))?.close();
}
