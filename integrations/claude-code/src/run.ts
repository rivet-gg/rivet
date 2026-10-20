import { spawn } from "node:child_process";
import { z } from "zod";

const NESTED_SESSION_VARIABLES = [
	"CLAUDECODE",
	"CLAUDE_CODE_ENTRYPOINT",
	"CLAUDE_CODE_SESSION_ID",
	"CLAUDE_CODE_CHILD_SESSION",
	"CLAUDE_CODE_SESSION_ATTENDED",
	"CLAUDE_CODE_MESSAGING_SOCKET",
	"CLAUDE_CODE_MESSAGING_TOKEN",
	"CLAUDE_PID",
];

export interface ClaudeCodeExecutable {
	command: string;
	args: string[];
}

export interface ClaudeCodeRequest {
	prompt: string;
	systemPrompt: string;
	tools?: readonly string[];
	model?: string;
	cwd?: string;
	resumeSessionId?: string;
	persistSession?: boolean;
	signal?: AbortSignal;
	executable?: ClaudeCodeExecutable;
}

export interface ClaudeCodeRun<Output> {
	output: Output;
	sessionId: string;
	costUsd: number;
	durationMs: number;
}

const cliResultSchema = z.object({
	type: z.literal("result"),
	subtype: z.string(),
	is_error: z.boolean(),
	result: z.string().optional(),
	structured_output: z.unknown().optional(),
	session_id: z.string(),
	total_cost_usd: z.number(),
	duration_ms: z.number(),
});

type CliResult = z.infer<typeof cliResultSchema>;

export function defaultExecutable(): ClaudeCodeExecutable {
	return { command: process.env.CLAUDE_CODE_BIN ?? "claude", args: [] };
}

export function runClaudeCode(request: ClaudeCodeRequest): Promise<ClaudeCodeRun<string>>;
export function runClaudeCode<Output>(
	request: ClaudeCodeRequest,
	outputSchema: z.ZodType<Output>,
): Promise<ClaudeCodeRun<Output>>;
export async function runClaudeCode<Output>(
	request: ClaudeCodeRequest,
	outputSchema?: z.ZodType<Output>,
): Promise<ClaudeCodeRun<Output | string>> {
	const cliResult = await invokeCli(request, buildArguments(request, outputSchema));

	if (cliResult.is_error) {
		throw new Error(`Claude Code run failed: ${cliResult.subtype}`);
	}

	return {
		output: outputSchema
			? outputSchema.parse(cliResult.structured_output)
			: (cliResult.result ?? ""),
		sessionId: cliResult.session_id,
		costUsd: cliResult.total_cost_usd,
		durationMs: cliResult.duration_ms,
	};
}

function buildArguments(
	request: ClaudeCodeRequest,
	outputSchema: z.ZodType | undefined,
): string[] {
	const tools = request.tools ?? [];
	const args = ["-p", "--output-format", "json", "--system-prompt", request.systemPrompt];

	if (!request.persistSession) {
		args.push("--no-session-persistence");
	}

	if (request.resumeSessionId) {
		args.push("--resume", request.resumeSessionId);
	}

	if (tools.length > 0) {
		args.push("--tools", ...tools, "--allowedTools", ...tools);
	} else {
		args.push("--tools", "");
	}

	if (outputSchema) {
		args.push(
			"--json-schema",
			JSON.stringify(z.toJSONSchema(outputSchema, { target: "draft-7" })),
		);
	}

	if (request.model) {
		args.push("--model", request.model);
	}

	return args;
}

function invokeCli(request: ClaudeCodeRequest, args: string[]): Promise<CliResult> {
	const executable = request.executable ?? defaultExecutable();

	return new Promise((resolve, reject) => {
		const child = spawn(executable.command, [...executable.args, ...args], {
			cwd: request.cwd,
			env: environmentWithoutNestedSession(),
			signal: request.signal,
			stdio: ["pipe", "pipe", "pipe"],
			// Without this, a windowed host on Windows flashes a console window per call.
			windowsHide: true,
		});

		let stdout = "";
		let stderr = "";
		child.stdout.setEncoding("utf8").on("data", (chunk: string) => {
			stdout += chunk;
		});
		child.stderr.setEncoding("utf8").on("data", (chunk: string) => {
			stderr += chunk;
		});

		child.on("error", (error) => {
			reject(
				new Error(
					`Could not start Claude Code (${executable.command}). Install it and run \`claude\` once to log in.`,
					{ cause: error },
				),
			);
		});

		child.on("close", (exitCode) => {
			const parsed = cliResultSchema.safeParse(parseJson(stdout));
			if (parsed.success) {
				resolve(parsed.data);
				return;
			}
			reject(
				new Error(
					`Claude Code exited with code ${exitCode} without a result: ${stderr.trim() || stdout.trim()}`,
				),
			);
		});

		child.stdin.end(request.prompt);
	});
}

function environmentWithoutNestedSession(): NodeJS.ProcessEnv {
	const environment = { ...process.env };
	for (const name of NESTED_SESSION_VARIABLES) {
		delete environment[name];
	}
	return environment;
}

function parseJson(text: string): unknown {
	try {
		return JSON.parse(text);
	} catch {
		return undefined;
	}
}
