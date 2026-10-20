import { Effect, Option, Schema } from "effect";
import { run } from "../exec.ts";
import { GitHub } from "./api.ts";

export const WORKFLOW_PATH = ".github/workflows/pipboy.yml";

const Runs = Schema.Struct({
	workflow_runs: Schema.Array(
		Schema.Struct({
			id: Schema.Number,
			path: Schema.String,
			event: Schema.String,
			head_branch: Schema.NullOr(Schema.String),
			head_sha: Schema.String,
			head_repository: Schema.Struct({ full_name: Schema.String }),
		}),
	),
});

const Artifacts = Schema.Struct({
	artifacts: Schema.Array(Schema.Struct({ name: Schema.String, expired: Schema.Boolean })),
});

type Run = Schema.Schema.Type<typeof Runs>["workflow_runs"][number];

/**
 * This workflow's completed push runs on `main` in this repository, newest
 * first. Nothing else is trusted as a baseline: a PR run can never supply one.
 * Empty until this workflow exists on main, when the API answers 404.
 */
const listMainRuns = Effect.fn("Baseline.listMainRuns")(function* (filter: string, pages: number) {
	const github = yield* GitHub;
	const runs: Array<Run> = [];
	for (let page = 1; page <= pages; page++) {
		const { workflow_runs } = yield* github
			.get(
				`/repos/${github.repository}/actions/workflows/pipboy.yml/runs?event=push&branch=main&status=completed${filter}&per_page=100&page=${page}`,
				Runs,
			)
			.pipe(
				Effect.catchIf(
					(error) => error._tag === "HttpClientError" && error.response?.status === 404,
					() => Effect.succeed({ workflow_runs: [] }),
				),
			);
		runs.push(
			...workflow_runs.filter(
				(r) =>
					r.event === "push" &&
					r.head_branch === "main" &&
					r.path === WORKFLOW_PATH &&
					r.head_repository.full_name === github.repository,
			),
		);
		if (workflow_runs.length < 100) break;
	}
	return runs;
});

const artifactNames = Effect.fn("Baseline.artifactNames")(function* (runId: number) {
	const github = yield* GitHub;
	const { artifacts } = yield* github.get(`/repos/${github.repository}/actions/runs/${runId}/artifacts?per_page=100`, Artifacts);
	return new Set(artifacts.filter((a) => !a.expired).map((a) => a.name));
});

const download = Effect.fn("Baseline.download")(function* (runId: number, artifact: string, outDir: string) {
	const github = yield* GitHub;
	yield* run("gh", ["run", "download", String(runId), "--repo", github.repository, "--name", artifact, "--dir", outDir]);
});

/** Downloads `artifact` from main's run for exactly `sha`. Returns the run id, or none. */
export const fetchBaseline = Effect.fn("Baseline.fetch")(function* (sha: string, artifact: string, outDir: string) {
	for (const r of (yield* listMainRuns(`&head_sha=${sha}`, 1)).filter((r) => r.head_sha === sha)) {
		if (!(yield* artifactNames(r.id)).has(artifact)) continue;
		yield* download(r.id, artifact, outDir);
		return Option.some(r.id);
	}
	return Option.none<number>();
});

/**
 * Downloads `artifacts` from the newest commit in `history` (newest first)
 * whose main run produced any of them. Main produces some artifacts only for
 * some commits, such as native builds only when Rust changed, so the result
 * names the commit the baseline came from.
 */
export const fetchNearestBaselines = Effect.fn("Baseline.fetchNearest")(function* (
	history: ReadonlyArray<string>,
	artifacts: ReadonlyArray<string>,
	outDir: string,
) {
	if (history.length === 0 || artifacts.length === 0) return Option.none<{ sha: string; found: ReadonlyArray<string> }>();
	const bySha = new Map<string, Array<Run>>();
	for (const r of yield* listMainRuns("", 3)) bySha.set(r.head_sha, [...(bySha.get(r.head_sha) ?? []), r]);
	for (const sha of history) {
		for (const r of bySha.get(sha) ?? []) {
			const available = yield* artifactNames(r.id);
			const found = artifacts.filter((a) => available.has(a));
			if (found.length === 0) continue;
			yield* Effect.forEach(found, (a) => download(r.id, a, outDir), { concurrency: 4, discard: true });
			return Option.some({ sha, found });
		}
	}
	return Option.none<{ sha: string; found: ReadonlyArray<string> }>();
});
