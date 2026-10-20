import { join } from "node:path";
import { Effect, FileSystem, Schema } from "effect";
import { compareFootprint } from "../compare/footprint.ts";
import { compareHygiene } from "../compare/hygiene.ts";
import { run } from "../exec.ts";
import { GitHub } from "../github/api.ts";
import { WORKFLOW_PATH } from "../github/baseline.ts";
import { readJobs } from "../github/timing.ts";
import { FootprintReport } from "../schema/footprint.ts";
import { HygieneReport } from "../schema/hygiene.ts";
import { NativeReport } from "../schema/native.ts";
import { type Attribute, attr, jobSpans, otlpDocument, runSpanId, span, toMillis, traceIdFor } from "./spans.ts";

const WorkflowRun = Schema.Struct({
	id: Schema.Number,
	name: Schema.NullOr(Schema.String),
	path: Schema.String,
	event: Schema.String,
	status: Schema.NullOr(Schema.String),
	conclusion: Schema.NullOr(Schema.String),
	run_attempt: Schema.Number,
	created_at: Schema.String,
	run_started_at: Schema.optionalKey(Schema.NullOr(Schema.String)),
	updated_at: Schema.String,
	html_url: Schema.String,
});
type WorkflowRun = Schema.Schema.Type<typeof WorkflowRun>;

const Runs = Schema.Struct({ total_count: Schema.Number, workflow_runs: Schema.Array(WorkflowRun) });
const Artifacts = Schema.Struct({
	artifacts: Schema.Array(Schema.Struct({ name: Schema.String, expired: Schema.Boolean })),
});

/** Every workflow run GitHub started for `sha`, across all workflows and events. */
/**
 * The workflows that build and test the code, by file. Bots and automation,
 * such as Claude review and docs sync, are left out of the trace.
 */
const TRACED_WORKFLOWS = new Set([
	".github/workflows/rust.yml",
	".github/workflows/frontend.yml",
	WORKFLOW_PATH,
	".github/workflows/publish.yaml",
	".github/workflows/swift.yml",
]);

const listRuns = Effect.fn("CommitTrace.listRuns")(function* (sha: string) {
	const github = yield* GitHub;
	const runs: Array<WorkflowRun> = [];
	for (let page = 1; ; page++) {
		const batch = yield* github.get(`/repos/${github.repository}/actions/runs?head_sha=${sha}&per_page=100&page=${page}`, Runs);
		runs.push(...batch.workflow_runs);
		if (batch.workflow_runs.length < 100 || runs.length >= batch.total_count) return runs;
	}
});

/**
 * Span attributes from a Pip-Boy run's reports, keyed by job name, so a
 * job's span carries the numbers it measured.
 */
const reportAttributes = Effect.fn("CommitTrace.reportAttributes")(function* (runId: number) {
	const github = yield* GitHub;
	const fs = yield* FileSystem.FileSystem;
	const byJob = new Map<string, Array<Attribute>>();
	const add = (job: string, attributes: ReadonlyArray<Attribute>) => byJob.set(job, [...(byJob.get(job) ?? []), ...attributes]);

	const { artifacts } = yield* github.get(`/repos/${github.repository}/actions/runs/${runId}/artifacts?per_page=100`, Artifacts);
	const reports = artifacts.filter((a) => a.name.startsWith("report-") && !a.expired);
	if (reports.length === 0) return byJob;
	const dir = yield* fs.makeTempDirectoryScoped({ prefix: "pipboy-trace-" });
	yield* run("gh", ["run", "download", String(runId), "--repo", github.repository, "--dir", dir, ...reports.flatMap((a) => ["--name", a.name])]);

	const read = <S extends Schema.Constraint>(path: string, schema: S) =>
		fs.readFileString(path).pipe(Effect.flatMap(Schema.decodeUnknownEffect(Schema.fromJsonString(schema))));
	for (const artifact of reports) {
		const artifactDir = reports.length === 1 ? dir : join(dir, artifact.name);
		for (const file of yield* fs.readDirectory(artifactDir)) {
			const path = join(artifactDir, file);
			if (file === "footprint.json") {
				const comparison = compareFootprint(yield* read(path, FootprintReport));
				for (const row of comparison.rows) {
					const pr = row.install[0]?.pr;
					if (!pr) continue;
					add("footprint", [attr(`rivet.footprint.install_bytes.${row.name}`, pr.bytes), attr(`rivet.footprint.packages.${row.name}`, pr.count)]);
				}
			} else if (file === "hygiene.json") {
				for (const row of compareHygiene(yield* read(path, HygieneReport)).rows) {
					add("hygiene", [attr(`rivet.hygiene.${row.check}.new`, row.added.length), attr(`rivet.hygiene.${row.check}.existing`, row.existing)]);
				}
			} else if (/^native-.+\.json$/.test(file)) {
				const { artifact: a } = yield* read(path, NativeReport);
				add(`native · ${a.target} · ${a.platform}`, [attr("rivet.native.bytes", a.bytes)]);
			}
		}
	}
	return byJob;
});

export class NoRunsForCommit extends Schema.TaggedError<NoRunsForCommit>()("NoRunsForCommit", {
	sha: Schema.String,
	repository: Schema.String,
}) {
	override get message() {
		return `no workflow runs found for ${this.sha} in ${this.repository}`;
	}
}

/**
 * One trace for the CI that ran on a commit: commit, then each traced workflow
 * run, its jobs, and their steps. Pip-Boy jobs also carry the sizes and
 * findings they measured.
 */
export const commitTrace = Effect.fn("CommitTrace.build")(function* (sha: string) {
	const github = yield* GitHub;
	const now = Date.now();
	const traceId = traceIdFor(github.repository, sha);
	// A push can start the same workflow twice for one commit; the newer run cancels the older.
	// Only the newest run of each traced workflow is kept.
	const newest = new Map<string, WorkflowRun>();
	for (const r of yield* listRuns(sha)) {
		if (!TRACED_WORKFLOWS.has(r.path)) continue;
		const kept = newest.get(r.path);
		if (kept === undefined || r.id > kept.id) newest.set(r.path, r);
	}
	const runs = [...newest.values()];
	if (runs.length === 0) return yield* new NoRunsForCommit({ sha, repository: github.repository });

	const perRun = yield* Effect.forEach(
		runs,
		(r) =>
			Effect.all({
				jobs: readJobs(r.id, r.run_attempt),
				attributes: r.path === WORKFLOW_PATH ? reportAttributes(r.id) : Effect.succeed(new Map<string, Array<Attribute>>()),
			}).pipe(Effect.map((details) => ({ run: r, ...details }))),
		{ concurrency: 8 },
	);

	const rootId = runSpanId(`commit:${sha}`);
	const runStart = (r: WorkflowRun) => toMillis(r.run_started_at, toMillis(r.created_at, now));
	const runEnd = (r: WorkflowRun) => (r.status === "completed" ? toMillis(r.updated_at, now) : now);
	const spans = [
		span({
			traceId,
			id: rootId,
			name: `commit ${sha.slice(0, 7)}`,
			startMs: Math.min(...runs.map(runStart)),
			endMs: Math.max(...runs.map(runEnd)),
			failed: runs.some((r) => r.conclusion === "failure"),
			attributes: [attr("vcs.ref.head.revision", sha), attr("vcs.repository.name", github.repository), attr("cicd.pipeline.run.count", runs.length)],
		}),
	];
	for (const { run: r, jobs, attributes } of perRun) {
		const id = runSpanId(r.id);
		spans.push(
			span({
				traceId,
				id,
				parent: rootId,
				name: r.name ?? r.path,
				startMs: runStart(r),
				endMs: runEnd(r),
				failed: r.conclusion === "failure",
				attributes: [
					attr("cicd.pipeline.name", r.name ?? r.path),
					attr("cicd.pipeline.run.id", r.id),
					attr("cicd.pipeline.run.url.full", r.html_url),
					attr("cicd.pipeline.result", r.conclusion ?? r.status ?? "unknown"),
					attr("cicd.pipeline.trigger.event", r.event),
					attr("cicd.pipeline.run.attempt", r.run_attempt),
				],
			}),
			...jobSpans(traceId, id, jobs, now, (job) => attributes.get(job.name) ?? []),
		);
	}
	return { runs: runs.length, document: otlpDocument(spans) };
});
