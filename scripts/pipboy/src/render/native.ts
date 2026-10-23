import { Option } from "effect";
import type { NativeRow } from "../compare/native.ts";
import { ICON, bytes, change, code } from "./format.ts";

/** `baseline` is the main commit the base sizes came from, if any. */
export function renderNative(
	rows: ReadonlyArray<NativeRow>,
	baseline: Option.Option<{ readonly sha: string; readonly isMergeBase: boolean }>,
): string {
	const source = Option.match(baseline, {
		onNone: () => "No native build on main to compare against yet.",
		onSome: ({ sha, isMergeBase }) =>
			isMergeBase
				? `Base: \`main@${sha.slice(0, 7)}\`, the merge base.`
				: `Base: \`main@${sha.slice(0, 7)}\`, the newest main commit with native builds before the merge base. Size changes merged to main since then show up here too.`,
	});
	const out = [
		"### 🦀 Binaries · publish Dockerfiles, `--release`",
		"",
		source,
		"",
		"| artifact | npm `@latest` | base | this PR | Δ vs base |",
		"|---|---:|---:|---:|---|",
	];
	for (const row of rows) {
		out.push(
			`| ${ICON[row.severity]}${code(`${row.target} ${row.platform}`)} | ${row.latest === undefined ? "—" : bytes(row.latest)} | ${row.base === undefined ? "—" : bytes(row.base)} | ${bytes(row.pr)} | ${row.base === undefined ? "—" : change(row.base, row.pr)} |`,
		);
	}
	for (const row of rows.filter((r) => r.crates.length > 0 && r.severity !== "ok")) {
		out.push(
			"",
			`<details><summary>${code(`${row.target} ${row.platform}`)} · crates that changed most (from symbol sizes, approximate)</summary>`,
			"",
			"| crate | base | this PR | Δ |",
			"|---|---:|---:|---|",
			...row.crates.map((c) => `| ${code(c.name)} | ${bytes(c.base)} | ${bytes(c.pr)} | ${change(c.base, c.pr)} |`),
			"",
			"</details>",
		);
	}
	return out.join("\n");
}
