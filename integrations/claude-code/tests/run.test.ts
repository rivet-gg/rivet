import { tmpdir } from "node:os";
import { realpathSync } from "node:fs";
import { describe, expect, test } from "vitest";
import { z } from "zod";
import { runClaudeCode } from "../src/run.ts";
import { argumentAfter, type FakeClaudeEcho, fakeClaude } from "./fake-claude-executable.ts";

async function runAndEcho(request: Partial<Parameters<typeof runClaudeCode>[0]> = {}) {
	const run = await runClaudeCode({
		prompt: "Hello",
		systemPrompt: "Be brief.",
		executable: fakeClaude(),
		...request,
	});
	return { run, echo: JSON.parse(run.output) as FakeClaudeEcho };
}

describe("runClaudeCode", () => {
	test("sends the prompt over stdin and returns the text result", async () => {
		const { run, echo } = await runAndEcho({ prompt: "What is Rivet?" });

		expect(echo.prompt).toBe("What is Rivet?");
		expect(echo.args).toEqual(expect.arrayContaining(["-p", "--output-format", "json"]));
		expect(argumentAfter(echo.args, "--system-prompt")).toBe("Be brief.");
		expect(run).toMatchObject({ costUsd: 0.01, durationMs: 5 });
	});

	test("enables and auto-approves only the requested tools", async () => {
		const { echo } = await runAndEcho({ tools: ["WebSearch", "WebFetch"] });

		const toolsAt = echo.args.indexOf("--tools");
		const allowedAt = echo.args.indexOf("--allowedTools");
		expect(echo.args.slice(toolsAt + 1, toolsAt + 3)).toEqual(["WebSearch", "WebFetch"]);
		expect(echo.args.slice(allowedAt + 1, allowedAt + 3)).toEqual(["WebSearch", "WebFetch"]);
	});

	test("disables all tools when none are requested", async () => {
		const { echo } = await runAndEcho();

		expect(argumentAfter(echo.args, "--tools")).toBe("");
		expect(echo.args).not.toContain("--allowedTools");
	});

	test("does not persist the session unless asked to", async () => {
		const { echo } = await runAndEcho();

		expect(echo.args).toContain("--no-session-persistence");
	});

	test("persists and resumes a session when asked to", async () => {
		const { echo } = await runAndEcho({ persistSession: true, resumeSessionId: "session-1" });

		expect(echo.args).not.toContain("--no-session-persistence");
		expect(argumentAfter(echo.args, "--resume")).toBe("session-1");
	});

	test("runs Claude Code in the requested working directory", async () => {
		const workingDirectory = realpathSync(tmpdir());
		const { echo } = await runAndEcho({ cwd: workingDirectory });

		expect(realpathSync(echo.cwd)).toBe(workingDirectory);
	});

	test("returns structured output validated against the schema", async () => {
		const run = await runClaudeCode(
			{
				prompt: "Plan",
				systemPrompt: "Plan.",
				executable: fakeClaude("--fake-structured-output", '{"subQuestions":["a","b"]}'),
			},
			z.object({ subQuestions: z.array(z.string()) }),
		);

		expect(run.output).toEqual({ subQuestions: ["a", "b"] });
	});

	test("passes the schema as JSON Schema draft-07, the dialect Claude Code validates", async () => {
		const run = await runClaudeCode(
			{ prompt: "Plan", systemPrompt: "Plan.", executable: fakeClaude("--fake-structured-echo") },
			z.object({ args: z.array(z.string()) }),
		);

		const jsonSchema = JSON.parse(argumentAfter(run.output.args, "--json-schema") ?? "{}");
		expect(jsonSchema.$schema).toBe("http://json-schema.org/draft-07/schema#");
	});

	test("rejects structured output that does not match the schema", async () => {
		await expect(
			runClaudeCode(
				{
					prompt: "Plan",
					systemPrompt: "Plan.",
					executable: fakeClaude("--fake-structured-output", '{"subQuestions":"oops"}'),
				},
				z.object({ subQuestions: z.array(z.string()) }),
			),
		).rejects.toThrow();
	});

	test("throws when Claude Code reports an error result", async () => {
		await expect(runAndEcho({ executable: fakeClaude("--fake-error") })).rejects.toThrow(
			"Claude Code run failed: error_during_execution",
		);
	});

	test("does not pass the parent Claude Code session to the child", async () => {
		process.env.CLAUDECODE = "1";
		try {
			const { echo } = await runAndEcho();
			expect(echo.nestedSession).toBeNull();
		} finally {
			delete process.env.CLAUDECODE;
		}
	});

	test("explains how to fix a missing Claude Code install", async () => {
		await expect(
			runAndEcho({ executable: { command: "claude-code-binary-that-does-not-exist", args: [] } }),
		).rejects.toThrow("Install it and run `claude` once to log in");
	});
});
