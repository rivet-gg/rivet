import { createHash } from "node:crypto";

export interface Step {
	readonly name: string;
	readonly number: number;
	readonly conclusion: string | null;
	readonly started_at: string | null;
	readonly completed_at: string | null;
}

export interface Job {
	readonly id: number;
	readonly name: string;
	readonly conclusion: string | null;
	readonly created_at?: string;
	readonly started_at: string | null;
	readonly completed_at: string | null;
	readonly steps?: ReadonlyArray<Step>;
}

export interface Attribute {
	readonly key: string;
	readonly value: { readonly stringValue: string } | { readonly intValue: string };
}

export interface Span {
	readonly traceId: string;
	readonly spanId: string;
	readonly parentSpanId?: string;
	readonly name: string;
	readonly kind: 1;
	readonly startTimeUnixNano: string;
	readonly endTimeUnixNano: string;
	readonly attributes: ReadonlyArray<Attribute>;
	readonly status: { readonly code: 1 | 2 };
}

const hex = (input: string, length: number) => createHash("sha256").update(input).digest("hex").slice(0, length);

/** One trace per commit: every workflow run for the commit lands in it, from any tool. */
export const traceIdFor = (repository: string, sha: string) => hex(`${repository}:${sha}`, 32);
export const runSpanId = (runId: number | string) => hex(`run:${runId}`, 16);

export const attr = (key: string, value: string | number): Attribute =>
	typeof value === "number"
		? { key, value: { intValue: String(Math.round(value)) } }
		: { key, value: { stringValue: value } };

/**
 * Short step names: `Run actions/checkout@<sha>` becomes `checkout`. Rendered
 * as code in comments, so GitHub never turns `owner/repo@sha` into a commit
 * link or `@name` into a mention.
 */
export function stepLabel(name: string): string {
	const action = name.match(/^(Post )?Run [\w.-]+\/([\w.-]+)(?:\/[\w./-]+)?@\S+$/);
	if (action) return `${action[1] ? "post " : ""}${action[2]}`;
	return name.split("\n")[0] ?? name;
}

export const toMillis = (iso: string | null | undefined, fallback: number) =>
	iso === null || iso === undefined ? fallback : Date.parse(iso);

export function span(fields: {
	traceId: string;
	id: string;
	parent?: string;
	name: string;
	startMs: number;
	endMs: number;
	failed?: boolean;
	attributes?: ReadonlyArray<Attribute>;
}): Span {
	const nanos = (ms: number) => `${BigInt(Math.round(ms)) * 1_000_000n}`;
	return {
		traceId: fields.traceId,
		spanId: fields.id,
		...(fields.parent ? { parentSpanId: fields.parent } : {}),
		name: fields.name,
		kind: 1,
		startTimeUnixNano: nanos(fields.startMs),
		endTimeUnixNano: nanos(Math.max(fields.startMs, fields.endMs)),
		attributes: fields.attributes ?? [],
		status: { code: fields.failed ? 2 : 1 },
	};
}

/** Job and step spans under `parent`. Running jobs and steps end at `now`. */
export function jobSpans(
	traceId: string,
	parent: string,
	jobs: ReadonlyArray<Job>,
	now: number,
	extraAttributes: (job: Job) => ReadonlyArray<Attribute> = () => [],
): Array<Span> {
	const spans: Array<Span> = [];
	for (const job of jobs) {
		if (job.started_at === null) continue;
		const jobId = hex(`job:${job.id}`, 16);
		const start = toMillis(job.started_at, now);
		spans.push(
			span({
				traceId,
				id: jobId,
				parent,
				name: job.name,
				startMs: start,
				endMs: toMillis(job.completed_at, now),
				failed: job.conclusion === "failure",
				attributes: [
					attr("cicd.pipeline.task.name", job.name),
					attr("cicd.pipeline.result", job.conclusion ?? "running"),
					// Time spent waiting for a runner before the job started.
					...(job.created_at ? [attr("cicd.pipeline.task.queued_ms", start - toMillis(job.created_at, start))] : []),
					...extraAttributes(job),
				],
			}),
		);
		for (const step of job.steps ?? []) {
			if (step.started_at === null) continue;
			spans.push(
				span({
					traceId,
					id: hex(`step:${job.id}:${step.number}`, 16),
					parent: jobId,
					name: stepLabel(step.name),
					startMs: toMillis(step.started_at, now),
					endMs: toMillis(step.completed_at, now),
					failed: step.conclusion === "failure",
					attributes: [attr("cicd.pipeline.task.step.name", step.name), attr("cicd.pipeline.result", step.conclusion ?? "running")],
				}),
			);
		}
	}
	return spans;
}

/** An OTLP/JSON trace export, as accepted by `POST /v1/traces`. */
export function otlpDocument(spans: ReadonlyArray<Span>) {
	return {
		resourceSpans: [
			{
				resource: { attributes: [attr("service.name", "rivet-ci")] },
				scopeSpans: [{ scope: { name: "pipboy" }, spans }],
			},
		],
	};
}
