import { Effect, Schema } from "effect";
import { GitHub } from "./api.ts";

export const MARKER = "<!-- pipboy -->";
const BOT = "github-actions[bot]";

const Pull = Schema.Struct({ head: Schema.Struct({ sha: Schema.String }) });
const Comments = Schema.Array(
	Schema.Struct({ id: Schema.Number, body: Schema.optionalKey(Schema.String), user: Schema.NullOr(Schema.Struct({ login: Schema.String })) }),
);

/**
 * Creates or updates the one Pip-Boy comment on a PR. Skips the write when
 * `headSha` is no longer the PR head, so a run for an older push can't
 * overwrite a newer result.
 */
export const upsertComment = Effect.fn("Comment.upsert")(function* (prNumber: number, headSha: string, body: string) {
	const github = yield* GitHub;
	const repo = github.repository;
	const pull = yield* github.get(`/repos/${repo}/pulls/${prNumber}`, Pull);
	if (pull.head.sha !== headSha) {
		yield* Effect.logInfo("skipping comment for a stale run").pipe(
			Effect.annotateLogs({ headSha, currentHead: pull.head.sha }),
		);
		return "stale" as const;
	}

	let existing: number | undefined;
	for (let page = 1; existing === undefined; page++) {
		const comments = yield* github.get(`/repos/${repo}/issues/${prNumber}/comments?per_page=100&page=${page}`, Comments);
		existing = comments.find((c) => c.user?.login === BOT && c.body?.startsWith(MARKER))?.id;
		if (comments.length < 100) break;
	}

	const content = `${MARKER}\n${body}`;
	if (existing === undefined) {
		yield* github.send("POST", `/repos/${repo}/issues/${prNumber}/comments`, { body: content });
		return "created" as const;
	}
	yield* github.send("PATCH", `/repos/${repo}/issues/comments/${existing}`, { body: content });
	return "updated" as const;
});
