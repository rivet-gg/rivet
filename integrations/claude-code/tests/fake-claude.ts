import { randomUUID } from "node:crypto";
import { readFileSync } from "node:fs";

const [fakeMode, fakeValue, ...cliArgs] = process.argv.slice(2);
const prompt = readFileSync(0, "utf8");
const resumeAt = cliArgs.indexOf("--resume");
const sessionId = resumeAt === -1 ? randomUUID() : cliArgs[resumeAt + 1];

const echo = {
	prompt,
	args: cliArgs,
	cwd: process.cwd(),
	nestedSession: process.env.CLAUDECODE ?? null,
};

function structuredOutput(): unknown {
	switch (fakeMode) {
		case "--fake-structured-output":
			return JSON.parse(fakeValue);
		case "--fake-structured-echo":
			return echo;
		default:
			return undefined;
	}
}

const isError = fakeMode === "--fake-error";

if (fakeMode === "--fake-delay") {
	await new Promise((resolve) => setTimeout(resolve, Number(fakeValue)));
}

console.log(
	JSON.stringify({
		type: "result",
		subtype: isError ? "error_during_execution" : "success",
		is_error: isError,
		result: JSON.stringify(echo),
		structured_output: structuredOutput(),
		session_id: sessionId,
		total_cost_usd: 0.01,
		duration_ms: 5,
	}),
);
