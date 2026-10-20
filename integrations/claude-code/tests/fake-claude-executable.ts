import { fileURLToPath } from "node:url";
import type { ClaudeCodeExecutable } from "../src/run.ts";

const FAKE_CLAUDE = fileURLToPath(new URL("./fake-claude.ts", import.meta.url));

export type FakeClaudeMode =
	| "--fake-text"
	| "--fake-delay"
	| "--fake-error"
	| "--fake-structured-output"
	| "--fake-structured-echo";

export interface FakeClaudeEcho {
	prompt: string;
	args: string[];
	cwd: string;
	nestedSession: string | null;
}

export function fakeClaude(mode: FakeClaudeMode = "--fake-text", value = ""): ClaudeCodeExecutable {
	return {
		command: process.execPath,
		args: ["--experimental-strip-types", "--no-warnings", FAKE_CLAUDE, mode, value],
	};
}

export function argumentAfter(args: string[], flag: string): string | undefined {
	const index = args.indexOf(flag);
	return index === -1 ? undefined : args[index + 1];
}
