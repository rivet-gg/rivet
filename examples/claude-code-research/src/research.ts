import { randomUUID } from "node:crypto";
import { createClient } from "rivetkit/client";
import type { ResearchProgress, registry } from "./actors.ts";

const question = process.argv.slice(2).join(" ").trim();
if (!question) {
	console.error('Usage: pnpm research "<your research question>"');
	process.exit(1);
}

const client = createClient<typeof registry>(
	process.env.RIVET_ENDPOINT ?? "http://127.0.0.1:6420",
);
const connection = client.researchCoordinator
	.getOrCreate([randomUUID()])
	.connect();

const finished = new Promise<void>((resolve) => {
	connection.on("progress", ({ status, message }: ResearchProgress) => {
		console.log(`[${status}] ${message}`);
		if (status === "done" || status === "failed") {
			resolve();
		}
	});
});

await connection.start(question);
await finished;

const result = await connection.getResult();
console.log(`\n${result.report ?? `Research failed: ${result.error}`}`);
console.log(
	`\nFindings: ${result.findings.length}, list-price cost: $${result.costUsd.toFixed(2)}`,
);

await connection.dispose();
process.exit(result.status === "done" ? 0 : 1);
