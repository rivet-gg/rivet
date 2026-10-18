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

/**
 * Downloads `artifact` from this workflow's `main` push run for `sha`. Only
 * pushes to `main` in this repository qualify, so a PR run can never supply
 * a baseline. Returns none when no such run or artifact exists.
 */
export const fetchBaseline = Effect.fn("Baseline.fetch")(function* (sha: string, artifact: string, outDir: string) {
	const github = yield* GitHub;
	// 404 until this workflow exists on main, so the first rollout has no baseline.
	const runs = yield* github
		.get(`/repos/${github.repository}/actions/workflows/pipboy.yml/runs?event=push&branch=main&head_sha=${sha}&per_page=20`, Runs)
		.pipe(
			Effect.catchIf(
				(error) => error._tag === "HttpClientError" && error.response?.status === 404,
				() => Effect.succeed({ workflow_runs: [] }),
			),
		);
	for (const run_ of runs.workflow_runs) {
		const trusted =
			run_.event === "push" &&
			run_.head_branch === "main" &&
			run_.head_sha === sha &&
			run_.path === WORKFLOW_PATH &&
			run_.head_repository.full_name === github.repository;
		if (!trusted) continue;
		const { artifacts } = yield* github.get(
			`/repos/${github.repository}/actions/runs/${run_.id}/artifacts?name=${encodeURIComponent(artifact)}`,
			Artifacts,
		);
		if (!artifacts.some((a) => a.name === artifact && !a.expired)) continue;
		yield* run("gh", ["run", "download", String(run_.id), "--repo", github.repository, "--name", artifact, "--dir", outDir]);
		return Option.some(run_.id);
	}
	return Option.none<number>();
});
