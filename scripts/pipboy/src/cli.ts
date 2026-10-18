import { join } from "node:path";
import { NodeHttpClient, NodeRuntime, NodeServices } from "@effect/platform-node";
import { Config, Console, Effect, FileSystem, Layer, Option, Schema } from "effect";
import { Command, Flag } from "effect/cli";
import { compareFootprint } from "./compare/footprint.ts";
import { REPO_ROOT } from "./config.ts";
import { inspectEntrypoints } from "./footprint/inspect.ts";
import { pack, readPackManifest } from "./footprint/pack.ts";
import * as Registry from "./footprint/registry.ts";
import { resolveSide } from "./footprint/resolve.ts";
import * as GitHubApi from "./github/api.ts";
import { fetchBaseline } from "./github/baseline.ts";
import { upsertComment } from "./github/comment.ts";
import { readTiming, renderTiming } from "./github/timing.ts";
import { renderFootprint } from "./render/footprint.ts";
import { JobResults, renderCheckSection } from "./render/status.ts";
import { FootprintReport, type FootprintSide } from "./schema/footprint.ts";

export class ChecksFailed extends Schema.TaggedError<ChecksFailed>()("ChecksFailed", {
	failures: Schema.Array(Schema.String),
}) {
	override get message() {
		return this.failures.join("\n");
	}
}

/** Reports larger than this are rejected before parsing. Artifacts are untrusted input. */
const MAX_REPORT_BYTES = 64 * 1024 * 1024;

const packCommand = Command.make(
	"pack",
	{
		out: Flag.String("out").pipe(Flag.withDescription("Directory for the tarballs and manifest.json")),
		root: Flag.String("root").pipe(Flag.withDefault(REPO_ROOT), Flag.withDescription("Checkout to pack, such as the merge base")),
	},
	({ out, root }) =>
		pack(root, out).pipe(Effect.flatMap((m) => Console.log(`packed ${Object.keys(m.tarballs).length} packages into ${out}`))),
).pipe(Command.withDescription("Build and pack publishable workspace packages the way npm publish would"));

const footprintCommand = Command.make(
	"footprint",
	{
		pr: Flag.String("pr").pipe(Flag.withDescription("Tarball directory of this change, from `pack`")),
		base: Flag.String("base").pipe(Flag.optional, Flag.withDescription("Tarball directory of the merge base")),
		out: Flag.String("out").pipe(Flag.withDescription("Where to write the JSON report")),
	},
	Effect.fn("Cli.footprint")(function* ({ pr, base, out }) {
		const fs = yield* FileSystem.FileSystem;
		// `@latest` is resolved for exactly the packages this change publishes.
		const roots = Object.keys((yield* readPackManifest(pr)).tarballs);

		const tarballSide = (side: "pr" | "base", dir: string) =>
			Effect.all([resolveSide({ side, tarballDir: dir }), inspectEntrypoints(dir)]).pipe(
				Effect.map(([resolved, entrypoints]): FootprintSide => ({ ...resolved, entrypoints })),
			);
		const sides = yield* Effect.scoped(
			Effect.all(
				[
					resolveSide({ side: "latest", roots }),
					tarballSide("pr", pr),
					...(Option.isSome(base) ? [tarballSide("base", base.value)] : []),
				],
				{ concurrency: "unbounded" },
			),
		);

		const report: FootprintReport = { schema: 1, check: "footprint", sides };
		yield* fs.writeFileString(out, JSON.stringify(report));
		const comparison = compareFootprint(report);
		yield* Console.log(renderFootprint(comparison));
		if (comparison.failures.length > 0) return yield* new ChecksFailed({ failures: comparison.failures });
	}, Effect.provide(Registry.layer)),
).pipe(Command.withDescription("Measure what users install, resolved fresh from the registry without downloading"));

const baselineCommand = Command.make(
	"baseline",
	{
		sha: Flag.String("sha").pipe(Flag.withDescription("Merge base commit")),
		artifact: Flag.String("artifact"),
		out: Flag.String("out"),
	},
	Effect.fn("Cli.baseline")(function* ({ sha, artifact, out }) {
		const found = yield* fetchBaseline(sha, artifact, out);
		yield* setOutput("found", String(Option.isSome(found)));
		yield* Console.log(Option.isSome(found) ? `baseline from run ${found.value}` : "no baseline on main for this commit");
	}, Effect.provide(GitHubApi.layer)),
).pipe(Command.withDescription("Download a baseline artifact from this workflow's main push run"));

const reportCommand = Command.make(
	"report",
	{
		reports: Flag.String("reports").pipe(Flag.withDescription("Directory holding the checks' JSON reports")),
	},
	Effect.fn("Cli.report")(function* ({ reports }) {
		const fs = yield* FileSystem.FileSystem;
		const github = yield* GitHubApi.GitHub;
		const env = yield* Config.all({
			// Empty for pushes and fork PRs, which get the run summary instead of a comment.
			prNumber: Config.String("PR_NUMBER").pipe(Config.withDefault("")),
			headSha: Config.String("HEAD_SHA"),
			summary: Config.option(Config.String("GITHUB_STEP_SUMMARY")),
			jobResults: Config.String("JOB_RESULTS").pipe(Config.withDefault("{}")),
		});
		const results = yield* Schema.decodeUnknownEffect(Schema.fromJsonString(JobResults))(env.jobResults);
		const section = (heading: string, job: string, skipReason: string, rendered: string | undefined) =>
			renderCheckSection({ heading, job, skipReason, rendered }, results, github.runUrl);

		const sections: Array<string> = [`## 📟 Pip-Boy · \`${env.headSha.slice(0, 7)}\``];
		const footprintPath = join(reports, "footprint.json");
		sections.push(
			section(
				"### 📦 Packages",
				"footprint",
				"this PR doesn't change any published package.",
				(yield* fs.exists(footprintPath))
					? renderFootprint(compareFootprint(yield* readReport(footprintPath, FootprintReport)))
					: undefined,
			),
		);

		const timing = yield* readTiming();
		sections.push(renderTiming(timing));
		const body = sections.join("\n\n");

		if (Option.isSome(env.summary)) {
			yield* fs.writeFileString(env.summary.value, `${body}\n`, { flag: "a" });
		}
		if (env.prNumber !== "") {
			const result = yield* upsertComment(Number(env.prNumber), env.headSha, body);
			yield* Console.log(`comment ${result}`);
		}
	}, Effect.provide(GitHubApi.layer)),
).pipe(Command.withDescription("Write the PR comment and run summary from the checks' reports"));

export class ReportTooLarge extends Schema.TaggedError<ReportTooLarge>()("ReportTooLarge", {
	path: Schema.String,
	bytes: Schema.Number,
}) {
	override get message() {
		return `${this.path} is ${this.bytes} bytes, over the ${MAX_REPORT_BYTES} byte limit for reports`;
	}
}

/** Reads a check's JSON report. Reports come from artifacts, so they're size-capped and schema-decoded. */
const readReport = Effect.fn("Cli.readReport")(function* <S extends Schema.Constraint>(path: string, schema: S) {
	const fs = yield* FileSystem.FileSystem;
	const bytes = Number((yield* fs.stat(path)).size);
	if (bytes > MAX_REPORT_BYTES) return yield* new ReportTooLarge({ path, bytes });
	return yield* Schema.decodeUnknownEffect(Schema.fromJsonString(schema))(yield* fs.readFileString(path));
});

/** Appends `name=value` to the step's outputs. A no-op outside GitHub Actions. */
const setOutput = Effect.fn("Cli.setOutput")(function* (name: string, value: string) {
	const path = yield* Config.option(Config.String("GITHUB_OUTPUT"));
	if (Option.isNone(path)) return;
	const fs = yield* FileSystem.FileSystem;
	yield* fs.writeFileString(path.value, `${name}=${value}\n`, { flag: "a" });
});

const cli = Command.make("pipboy").pipe(
	Command.withSubcommands([packCommand, footprintCommand, baselineCommand, reportCommand]),
);

Command.run(cli, { version: "0.0.0" }).pipe(
	Effect.provide(Layer.mergeAll(NodeServices.layer, NodeHttpClient.layerUndici)),
	NodeRuntime.runMain,
);
