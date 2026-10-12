import type { OpenCode } from "@opencode/sdk/effect";
import { Effect } from "effect";
import { describe, expect, it } from "vitest";
import { createHost } from "../dist/host.js";
import { sqliteFixture } from "./sqlite-fixture.js";

describe("embedded OpenCode host", () => {
	it("keeps a session in actor SQLite across a host restart", async () => {
		const { db } = sqliteFixture();
		try {
			const first = await createHost(db, { models: { fetch: false } });
			let sessionID: string;
			try {
				const session = await Effect.runPromise(
					first.client.session.create({
						location: {
							directory: process.cwd() as NonNullable<
								NonNullable<
									Parameters<
										OpenCode.Interface["session"]["create"]
									>[0]
								>["location"]
							>["directory"],
						},
					}),
				);
				sessionID = session.id;
			} finally {
				await first.close();
			}
			const second = await createHost(db, { models: { fetch: false } });
			try {
				const restored = await Effect.runPromise(
					second.client.session.get({
						sessionID: sessionID as Parameters<
							OpenCode.Interface["session"]["get"]
						>[0]["sessionID"],
					}),
				);
				expect(restored.id).toBe(sessionID);
			} finally {
				await second.close();
			}
		} finally {
			await db.close();
		}
	}, 60_000);
});
