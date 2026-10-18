import { Effect, Schema, Stream } from "effect";
import { ChildProcess, ChildProcessSpawner } from "effect/process";

export class CommandFailed extends Schema.TaggedError<CommandFailed>()("CommandFailed", {
	command: Schema.String,
	exitCode: Schema.Number,
	stderr: Schema.String,
}) {
	override get message() {
		return `\`${this.command}\` exited with ${this.exitCode}\n${this.stderr}`;
	}
}

export interface RunOptions {
	readonly cwd?: string;
	readonly env?: Record<string, string | undefined>;
}

/** Runs a command to completion and returns its stdout. Fails on a non-zero exit code. */
export const run = Effect.fn("Exec.run")(function* (
	command: string,
	args: ReadonlyArray<string>,
	options: RunOptions = {},
) {
	const spawner = yield* ChildProcessSpawner.ChildProcessSpawner;
	return yield* Effect.scoped(
		Effect.gen(function* () {
			const handle = yield* spawner.spawn(
				ChildProcess.make(command, args, { cwd: options.cwd, env: options.env, extendEnv: true }),
			);
			const [stdout, stderr, exitCode] = yield* Effect.all(
				[
					Stream.mkString(Stream.decodeText(handle.stdout)),
					Stream.mkString(Stream.decodeText(handle.stderr)),
					handle.exitCode,
				],
				{ concurrency: "unbounded" },
			);
			if (exitCode !== 0) {
				return yield* new CommandFailed({
					command: [command, ...args].join(" "),
					exitCode,
					stderr: stderr.slice(-4000),
				});
			}
			return stdout;
		}),
	);
});

/**
 * Environment for running npm. Package managers inject their own `npm_config_*`
 * settings into child processes (pnpm passes `catalog`, `reporter`, and others),
 * which change npm's behavior. Those are removed; registry, cache, user config,
 * and auth settings are kept.
 */
export function npmEnv(): Record<string, string | undefined> {
	const keep = /^npm_config_(registry|cache|userconfig|globalconfig|_auth.*|.*:_auth.*|\/\/.*)$/i;
	return Object.fromEntries(
		Object.keys(process.env)
			.filter((key) => /^npm_/i.test(key) && !keep.test(key))
			.map((key) => [key, undefined]),
	);
}
