import { BLOCKING, type HygieneComparison } from "../compare/hygiene.ts";
import { code } from "./format.ts";

const MAX_LISTED = 20;

export function renderHygiene(c: HygieneComparison): string {
	const out = [
		"### 🧹 Rust dependencies · `cargo deny` + `cargo shear`",
		"",
		"| check | new in this PR | fixed | existing on base |",
		"|---|---|---:|---:|",
	];
	for (const row of c.rows) {
		const icon = row.added.length === 0 ? "✅" : BLOCKING.has(row.check) ? "❌" : "🟡";
		out.push(`| ${row.check} | ${icon} ${row.added.length} | ${row.fixed} | ${row.existing} |`);
	}
	if (!c.hasBase) out.push("", "No base to compare against: every finding counts as existing.");

	const added = c.rows.flatMap((row) => row.added.map((f) => ({ ...f, check: row.check })));
	if (added.length > 0) {
		out.push("", "| new finding | via |", "|---|---|");
		for (const f of added.slice(0, MAX_LISTED)) out.push(`| ${code(f.title)} | ${code(f.via)} |`);
		if (added.length > MAX_LISTED) out.push(`| … and ${added.length - MAX_LISTED} more | |`);
	}
	return out.join("\n");
}
