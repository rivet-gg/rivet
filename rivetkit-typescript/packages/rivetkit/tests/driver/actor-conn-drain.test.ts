import { expect, test } from "vitest";
import type { ActorError } from "@/client/errors";
import { createClient } from "@/client/mod";
import { CONN_DRAIN_DISCONNECT_MARKER } from "../../fixtures/driver-test-suite/conn-drain";
import type { registry } from "../../fixtures/driver-test-suite/registry-static";
import { describeDriverMatrix } from "./shared-matrix";

describeDriverMatrix(
	"Connection Drain",
	(driverTestConfig) => {
		test("closes connections with ws.going_away and runs onDisconnect once", async (c) => {
			const runtime = await driverTestConfig.start({
				env: { RIVETKIT_TEST_DRAIN_ON_SIGTERM: "1" },
			});
			const client = createClient<typeof registry>({
				endpoint: runtime.endpoint,
				namespace: runtime.namespace,
				poolName: runtime.runnerName,
				encoding: driverTestConfig.encoding,
				disableMetadataLookup: true,
			});
			c.onTestFinished(async () => {
				await client.dispose();
				await runtime.cleanup();
			});

			const key = crypto.randomUUID();
			const conn = client.connDrainActor.getOrCreate([key]).connect();
			const errors: ActorError[] = [];
			conn.onError((error) => errors.push(error));
			const closed = new Promise<void>((resolve) =>
				conn.onClose(resolve),
			);
			expect(await conn.ping()).toBe("pong");

			await runtime.cleanup();
			await closed;

			expect(
				errors.map((error) => `${error.group}.${error.code}`),
			).toEqual(["ws.going_away"]);
			const output = runtime.getRuntimeOutput?.() ?? "";
			const hookRuns =
				output.split(`${CONN_DRAIN_DISCONNECT_MARKER} ${key}`).length - 1;
			expect(hookRuns).toBe(1);
		});
	},
	{
		encodings: ["bare"],
		runtimes: ["native"],
		sqliteBackends: ["local"],
	},
);
