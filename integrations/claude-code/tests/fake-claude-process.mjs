import { randomUUID } from "node:crypto";
import { createInterface } from "node:readline";

const args = process.argv.slice(2);
const sessionId = args.find((arg) => arg.startsWith("--resume="))?.slice("--resume=".length) ?? randomUUID();
const failWith = process.env.FAKE_CLAUDE_FAIL_WITH;
const delayMs = Number(process.env.FAKE_CLAUDE_DELAY_MS ?? 0);
let initializeRequest = {};

function send(message) {
	process.stdout.write(`${JSON.stringify(message)}\n`);
}

function promptText(content) {
	return typeof content === "string" ? content : content.map((block) => block.text).join("");
}

function echo(prompt) {
	return {
		prompt,
		systemPrompt: initializeRequest.systemPrompt ?? null,
		jsonSchema: initializeRequest.jsonSchema ?? null,
		args,
		cwd: process.cwd(),
		parentSession: process.env.CLAUDECODE ?? null,
	};
}

function structuredOutput(prompt) {
	if (process.env.FAKE_CLAUDE_ECHO_AS_STRUCTURED_OUTPUT) {
		return echo(prompt);
	}
	const configured = process.env.FAKE_CLAUDE_STRUCTURED_OUTPUT;
	return configured === undefined ? undefined : JSON.parse(configured);
}

function result(prompt) {
	const common = {
		type: "result",
		session_id: sessionId,
		total_cost_usd: 0.01,
		duration_ms: 5,
		duration_api_ms: 4,
		num_turns: 1,
		stop_reason: null,
		usage: {},
		modelUsage: {},
		permission_denials: [],
		uuid: randomUUID(),
	};
	if (failWith) {
		return { ...common, subtype: failWith, is_error: true, errors: ["fake failure"] };
	}
	return {
		...common,
		subtype: "success",
		is_error: false,
		result: JSON.stringify(echo(prompt)),
		structured_output: structuredOutput(prompt),
	};
}

process.stdin.on("end", () => process.exit(0));

for await (const line of createInterface({ input: process.stdin })) {
	const message = JSON.parse(line);
	if (message.type === "control_request") {
		initializeRequest = message.request;
		send({ type: "control_response", response: { subtype: "success", request_id: message.request_id, response: {} } });
	}
	if (message.type === "user") {
		await new Promise((resolve) => setTimeout(resolve, delayMs));
		send(result(promptText(message.message.content)));
	}
}
