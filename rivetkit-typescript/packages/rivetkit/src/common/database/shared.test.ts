import { DatabaseSync } from "node:sqlite";
import { setImmediate } from "node:timers/promises";
import { expect, test, vi } from "vitest";
import type {
	SqliteDatabase,
	SynchronousSqliteTransactionDatabase,
} from "./config";
import { runSqliteTransactionSync } from "./shared";

test.each([
	"immediate",
	"after-await",
	"callable-thenable",
])("rejects %s callback promises without an unhandled rejection and rolls back real SQL", async (kind) => {
	const sqlite = new DatabaseSync(":memory:");
	sqlite.exec("CREATE TABLE items(value INTEGER)");
	// Only transaction boundaries are needed by the production callback helper.
	const database = {
		beginTransactionSync() {
			sqlite.exec("BEGIN");
			return {
				commitSync() {
					sqlite.exec("COMMIT");
					return null;
				},
				rollbackSync() {
					sqlite.exec("ROLLBACK");
				},
			} as SynchronousSqliteTransactionDatabase;
		},
	} as SqliteDatabase;
	const unhandled = vi.fn();
	process.on("unhandledRejection", unhandled);
	try {
		expect(() =>
			runSqliteTransactionSync(database, () => {
				const result = (async () => {
					sqlite.exec("INSERT INTO items VALUES (1)");
					if (kind === "after-await") await Promise.resolve();
					throw new Error("callback rejected");
				})();
				return kind === "callable-thenable"
					? Object.assign(() => {}, {
							then: result.then.bind(result),
						})
					: result;
			}),
		).toThrow("must not return a promise");
		expect(sqlite.prepare("SELECT value FROM items").all()).toEqual([]);
		sqlite.exec("INSERT INTO items VALUES (2)");
		// Node reports unhandled rejections before the next event-loop turn.
		await setImmediate();
		expect(unhandled).not.toHaveBeenCalled();
		expect(sqlite.prepare("SELECT value FROM items").all()).toEqual([
			{ value: 2 },
		]);
	} finally {
		process.off("unhandledRejection", unhandled);
		sqlite.close();
	}
});
