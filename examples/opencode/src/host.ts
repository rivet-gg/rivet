import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Database } from "@opencode/core/database/database";
import { EffectDrizzleSqlite } from "@opencode/core/database/drizzle";
import { DatabaseMigration } from "@opencode/core/database/migration";
import { SessionRestart } from "@opencode/core/session/execution/restart";
import { OpenCode } from "@opencode/sdk/effect";
import { Global } from "@opencode/util/global";
import { Effect, Exit, Layer, Scope, type Stream } from "effect";
import { type ActorSqlite, sqliteLayer } from "./sqlite.js";

export type OpenCodeOptions = Omit<
	OpenCode.CreateOptions,
	"database" | "events" | "fs" | "instances" | "workspaceProviders"
>;
export type OpenCodeEvent = Stream.Success<
	ReturnType<OpenCode.Interface["event"]["subscribe"]>
>;

// OpenCode uses the actor's SQLite database for sessions, events, and migrations.
export async function createHost(
	database: ActorSqlite,
	options: OpenCodeOptions = {},
) {
	const directory = await mkdtemp(join(tmpdir(), "rivet-opencode-"));
	const scope = await Effect.runPromise(Scope.make());
	const close = async () => {
		try {
			await Effect.runPromise(Scope.close(scope, Exit.void));
		} finally {
			await rm(directory, { recursive: true, force: true });
		}
	};
	try {
		const global = Global.layerWith(
			Object.fromEntries(
				[
					"home",
					"data",
					"cache",
					"config",
					"state",
					"tmp",
					"bin",
					"log",
					"repos",
				].map((key) => [key, join(directory, key)]),
			),
		);
		const databaseLayer = Layer.effect(
			Database.Service,
			Effect.gen(function* () {
				const db = yield* EffectDrizzleSqlite.makeWithDefaults();
				yield* db.run("PRAGMA foreign_keys = ON");
				yield* DatabaseMigration.apply(db);
				return { db };
			}).pipe(Effect.orDie),
		).pipe(Layer.provide(sqliteLayer(database)), Layer.provide(global));
		let resume = (): Promise<void> =>
			Promise.reject(new Error("OpenCode recovery was not initialized"));
		const recovery = SessionRestart.node.mapLayer((layer) =>
			Layer.effect(
				SessionRestart.Service,
				Effect.map(SessionRestart.Service, (service) => {
					resume = () =>
						Effect.runPromise(service.resumeSuspendedSessions);
					// Resume after the actor has installed its event listener.
					return { resumeSuspendedSessions: Effect.void };
				}),
			).pipe(Layer.provide(layer)),
		);
		const client = await Effect.runPromise(
			OpenCode.create(
				{
					...options,
					events: { persist: true },
					fs: { filewatcher: false, fff: false },
				},
				{
					overrides: [
						Global.node.replace(global),
						Database.node.replace(databaseLayer),
						SessionRestart.node.replace(recovery),
					],
				},
			).pipe(Scope.provide(scope)),
		);
		return { client, scope, close, resume };
	} catch (error) {
		await close();
		throw error;
	}
}

export type OpenCodeHost = Awaited<ReturnType<typeof createHost>>;
