import { fileURLToPath } from "node:url";

export const FAKE_CLAUDE_CODE = fileURLToPath(new URL("./fake-claude-process.mjs", import.meta.url));

export interface FakeClaudeEcho {
	prompt: string;
	systemPrompt: string[] | null;
	jsonSchema: Record<string, unknown> | null;
	args: string[];
	cwd: string;
	parentSession: string | null;
}

export interface FakeClaudeBehavior {
	failWith?: "error_during_execution";
	structuredOutput?: unknown;
	echoAsStructuredOutput?: boolean;
	delayMs?: number;
}

export async function withFakeClaude<T>(behavior: FakeClaudeBehavior, work: () => Promise<T>): Promise<T> {
	const variables: Record<string, string | undefined> = {
		FAKE_CLAUDE_FAIL_WITH: behavior.failWith,
		FAKE_CLAUDE_STRUCTURED_OUTPUT: behavior.structuredOutput === undefined ? undefined : JSON.stringify(behavior.structuredOutput),
		FAKE_CLAUDE_ECHO_AS_STRUCTURED_OUTPUT: behavior.echoAsStructuredOutput ? "1" : undefined,
		FAKE_CLAUDE_DELAY_MS: behavior.delayMs?.toString(),
	};
	Object.assign(process.env, Object.fromEntries(Object.entries(variables).filter(([, value]) => value !== undefined)));
	try {
		return await work();
	} finally {
		for (const name of Object.keys(variables)) {
			delete process.env[name];
		}
	}
}

export function optionValue(args: string[], option: string): string | undefined {
	return args.find((arg) => arg.startsWith(`${option}=`))?.slice(option.length + 1);
}
