import { Effect, Schema } from "effect";
import { code } from "../render/format.ts";
import { GitHub } from "./api.ts";

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

/** Job and step timings of the current run, including still-running jobs. */
export const readTiming = Effect.fn("Timing.read")(function* () {
	const github = yield* GitHub;
	const { jobs } = yield* github.get(
		`/repos/${github.repository}/actions/runs/${github.runId}/attempts/${github.runAttempt}/jobs?per_page=100`,
		Jobs,
	);
	return { jobs: jobs.filter((job) => job.started_at !== null), now: Date.now() } satisfies Timing;
});

/**
 * Short step names: `Run actions/checkout@<sha>` becomes `checkout`. Rendered
 * as code, so GitHub never turns `owner/repo@sha` into a commit link or `@name`
 * into a mention.
 */
export function stepLabel(name: string): string {
	const action = name.match(/^(Post )?Run [\w.-]+\/([\w.-]+)(?:\/[\w./-]+)?@\S+$/);
	if (action) return `${action[1] ? "post " : ""}${action[2]}`;
	return name.split("\n")[0] ?? name;
}

const ms = (iso: string | null, fallback: number) => (iso === null ? fallback : Date.parse(iso));

function duration(totalMs: number): string {
	const s = Math.max(0, Math.round(totalMs / 1000));
	return s >= 60 ? `${Math.floor(s / 60)}m${String(s % 60).padStart(2, "0")}s` : `${s}s`;
}

/** Comment section: wall time per job and its slowest step. Job names link to their logs. */
export function renderTiming(timing: Timing): string {
	const lines = ["### ⏱ Timing", "", "| job | wall time | slowest step |", "|---|---:|---|"];
	for (const job of timing.jobs) {
		const start = ms(job.started_at, timing.now);
		const slowest = [...(job.steps ?? [])]
			.map((s) => ({ name: s.name, ms: ms(s.completed_at, timing.now) - ms(s.started_at, timing.now) }))
			.sort((a, b) => b.ms - a.ms)[0];
		const name = job.html_url === undefined ? code(job.name) : `[${code(job.name)}](${job.html_url})`;
		if (job.conclusion === "skipped") {
			lines.push(`| ⏭ ${name} | — | skipped |`);
			continue;
		}
		const icon = job.conclusion === "failure" ? "❌ " : job.conclusion === "cancelled" ? "⊘ " : "";
		lines.push(
			`| ${icon}${name} | ${duration(ms(job.completed_at, timing.now) - start)} | ${slowest ? `${code(stepLabel(slowest.name))} · ${duration(slowest.ms)}` : "—"} |`,
		);
	}
	return lines.join("\n");
}
