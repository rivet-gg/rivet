import type { HygieneCheck, HygieneFinding, HygieneReport } from "../schema/hygiene.ts";

export const HYGIENE_CHECKS: ReadonlyArray<HygieneCheck> = ["advisories", "licenses", "sources", "bans", "unused"];

/** New findings in these checks fail the job. Bans and unused dependencies are reported only. */
export const BLOCKING: ReadonlySet<HygieneCheck> = new Set(["advisories", "licenses", "sources"]);

export interface HygieneComparison {
	readonly hasBase: boolean;
	readonly rows: ReadonlyArray<{
		readonly check: HygieneCheck;
		readonly added: ReadonlyArray<HygieneFinding>;
		readonly fixed: number;
		readonly existing: number;
	}>;
	readonly failures: ReadonlyArray<string>;
}

/** Findings in the PR that the base doesn't have. Existing debt never fails a PR. */
export function compareHygiene(report: HygieneReport): HygieneComparison {
	const pr = report.sides.find((side) => side.side === "pr")?.findings ?? [];
	const base = report.sides.find((side) => side.side === "base")?.findings;
	const baseKeys = new Set((base ?? []).map((f) => f.key));
	const prKeys = new Set(pr.map((f) => f.key));

	const rows = HYGIENE_CHECKS.map((check) => {
		const inPr = pr.filter((f) => f.check === check);
		const added = base === undefined ? [] : inPr.filter((f) => !baseKeys.has(f.key));
		return {
			check,
			added,
			fixed: (base ?? []).filter((f) => f.check === check && !prKeys.has(f.key)).length,
			existing: inPr.length - added.length,
		};
	});
	const failures = rows
		.filter((row) => BLOCKING.has(row.check))
		.flatMap((row) => row.added.map((f) => `new ${row.check} finding: ${f.title}`));
	return { hasBase: base !== undefined, rows, failures };
}
