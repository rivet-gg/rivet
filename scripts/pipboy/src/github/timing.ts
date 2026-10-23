import { Effect, Schema } from "effect";
import { code } from "../render/format.ts";
import { stepLabel, toMillis } from "../trace/spans.ts";
import { CurrentRun, GitHub } from "./api.ts";

const Step = Schema.Struct({
	name: Schema.String,
	number: Schema.Number,
	conclusion: Schema.NullOr(Schema.String),
	started_at: Schema.NullOr(Schema.String),
	completed_at: Schema.NullOr(Schema.String),
});

const Job = Schema.Struct({
	id: Schema.Number,
	name: Schema.String,
	conclusion: Schema.NullOr(Schema.String),
	created_at: Schema.optionalKey(Schema.String),
	started_at: Schema.NullOr(Schema.String),
	completed_at: Schema.NullOr(Schema.String),
	html_url: Schema.optionalKey(Schema.String),
	steps: Schema.optionalKey(Schema.Array(Step)),
});
type Job = Schema.Schema.Type<typeof Job>;

const Jobs = Schema.Struct({ jobs: Schema.Array(Job) });

export interface Timing {
	readonly jobs: ReadonlyArray<Job>;
	/** Wall clock time the jobs were read. Running jobs end here. */
	readonly now: number;
}

/** Every job of one run attempt, including still-running jobs. */
export const readJobs = Effect.fn("Timing.readJobs")(function* (runId: number | string, attempt: number | string) {
	const github = yield* GitHub;
	const jobs: Array<Job> = [];
	for (let page = 1; ; page++) {
		const batch = yield* github.get(
			`/repos/${github.repository}/actions/runs/${runId}/attempts/${attempt}/jobs?per_page=100&page=${page}`,
			Jobs,
		);
		jobs.push(...batch.jobs);
		if (batch.jobs.length < 100) return jobs;
	}
});

/** Job and step timings of the current run. */
export const readTiming = Effect.fn("Timing.read")(function* () {
	const run = yield* CurrentRun;
	const jobs = yield* readJobs(run.runId, run.runAttempt);
	return { jobs: jobs.filter((job) => job.started_at !== null), now: Date.now() } satisfies Timing;
});

function duration(totalMs: number): string {
	const s = Math.max(0, Math.round(totalMs / 1000));
	return s >= 60 ? `${Math.floor(s / 60)}m${String(s % 60).padStart(2, "0")}s` : `${s}s`;
}

/** Comment section: wall time per job and its slowest step. Job names link to their logs. */
export function renderTiming(timing: Timing): string {
	const lines = ["### ⏱ Timing", "", "| job | wall time | slowest step |", "|---|---:|---|"];
	for (const job of timing.jobs) {
		const start = toMillis(job.started_at, timing.now);
		const slowest = [...(job.steps ?? [])]
			.map((s) => ({ name: s.name, ms: toMillis(s.completed_at, timing.now) - toMillis(s.started_at, timing.now) }))
			.sort((a, b) => b.ms - a.ms)[0];
		const name = job.html_url === undefined ? code(job.name) : `[${code(job.name)}](${job.html_url})`;
		if (job.conclusion === "skipped") {
			lines.push(`| ⏭ ${name} | — | skipped |`);
			continue;
		}
		const icon = job.conclusion === "failure" ? "❌ " : job.conclusion === "cancelled" ? "⊘ " : "";
		lines.push(
			`| ${icon}${name} | ${duration(toMillis(job.completed_at, timing.now) - start)} | ${slowest ? `${code(stepLabel(slowest.name))} · ${duration(slowest.ms)}` : "—"} |`,
		);
	}
	return lines.join("\n");
}
