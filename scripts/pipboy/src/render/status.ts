import { Schema } from "effect";

const JobResult = Schema.Literals(["success", "failure", "cancelled", "skipped"]);

/** `toJSON(needs)` from the workflow: each needed job's result. */
export const JobResults = Schema.Record(Schema.String, Schema.Struct({ result: JobResult }));
export type JobResults = Schema.Schema.Type<typeof JobResults>;

const PAST: Record<Schema.Schema.Type<typeof JobResult>, string> = {
	success: "succeeded",
	failure: "failed",
	cancelled: "was cancelled",
	skipped: "was skipped",
};

export interface CheckSection {
	/** Section heading, such as `### 📦 Packages`. */
	readonly heading: string;
	/** Workflow job that produces the report. */
	readonly job: string;
	/** Why the job is skipped when nothing relevant changed, completing "Skipped: …". */
	readonly skipReason: string;
	/** The rendered report, when the job produced one. Starts with its own heading. */
	readonly rendered: string | undefined;
}

/**
 * The comment section for one check, from the job's actual result. A missing
 * report is never read as "skipped": a failed job says it failed and links the
 * run, even when it wrote no report.
 */
export function renderCheckSection(section: CheckSection, results: JobResults, runUrl: string): string {
	const run = `See the [run](${runUrl}).`;
	const say = (line: string) => `${section.heading}\n\n${line}`;
	// Every check depends on `changes`, so a check skipped after `changes` failed never had a chance to run.
	const changes = results.changes?.result;
	const result = results[section.job]?.result;
	if (result === undefined) return say(`❌ The \`${section.job}\` job is missing from the workflow's \`needs\`. ${run}`);

	switch (result) {
		case "skipped":
			return changes === "success"
				? say(`Skipped: ${section.skipReason}`)
				: say(`❌ Not run: the \`changes\` job ${changes === undefined ? "is missing" : PAST[changes]}. ${run}`);
		case "cancelled":
			return say(`⊘ The \`${section.job}\` job was cancelled.`);
		case "failure": {
			const failed = `❌ The \`${section.job}\` job failed. ${run}`;
			return section.rendered === undefined ? say(`${failed} It produced no report.`) : insertAfterHeading(section.rendered, failed);
		}
		case "success":
			return section.rendered ?? say(`❌ The \`${section.job}\` job succeeded but produced no report. ${run}`);
	}
}

function insertAfterHeading(rendered: string, line: string): string {
	const end = rendered.indexOf("\n");
	return end === -1 ? `${rendered}\n\n${line}` : `${rendered.slice(0, end)}\n\n${line}${rendered.slice(end)}`;
}
