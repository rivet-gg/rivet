import type { EntrypointRow, FootprintComparison, PackageRow } from "../compare/footprint.ts";
import { GATED_PLATFORM } from "../config.ts";
import { ICON, bytes, change, code, count } from "./format.ts";

export function renderFootprint(c: FootprintComparison): string {
	const ref = c.reference;
	const out: Array<string> = [
		`### 📦 Packages · fresh \`npm install\` resolve, no lockfile · this PR vs ${ref === "base" ? "base" : "`@latest`"}`,
		"",
	];
	if (c.latestOverBudget.length > 0) {
		out.push(
			`> 🔴 **Registry drift:** ${c.latestOverBudget.map((n) => code(`${n}@latest`)).join(", ")} is already over budget for anyone installing it today. This PR didn't cause it.`,
			"",
		);
	}

	const changed = c.rows.filter(isNotable);
	if (changed.length === 0) {
		out.push(`No notable install size, dependency, or entrypoint changes across ${c.rows.length} package${c.rows.length === 1 ? "" : "s"}.`, "");
	} else {
		out.push(...summaryTable(changed, ref), "");
		for (const row of changed) out.push(...details(row, ref));
	}

	out.push(
		`<details><summary>All ${c.rows.length} packages · ${GATED_PLATFORM.id}</summary>`,
		"",
		...summaryTable(c.rows, ref),
		"",
		"</details>",
	);
	return out.join("\n");
}

/**
 * A package is worth showing when this PR changed something meaningful (a size
 * crossing its threshold, or a different set of dependencies, install scripts,
 * imports, or exports) or when it's over budget, since its details explain why.
 * Byte-level noise, such as build paths embedded in output, is not.
 */
function isNotable(row: PackageRow): boolean {
	return (
		row.severity !== "ok" ||
		row.budget?.over.pr === true ||
		row.added.length + row.removed.length + row.installScriptsAdded.length > 0 ||
		row.entrypoints.some((e) => e.severity !== "ok" || hasSurfaceChange(e))
	);
}

function summaryTable(rows: ReadonlyArray<PackageRow>, ref: "latest" | "base" | "pr"): Array<string> {
	const lines = [
		`| package (${GATED_PLATFORM.id}) | \`@latest\` | base | this PR | Δ vs ${ref} | packages |`,
		"|---|---:|---:|---:|---|---:|",
	];
	for (const row of rows) {
		const g = row.install[0];
		const r = g?.[ref];
		const budget = row.budget && row.budget.over.pr ? ` · over ${bytes(row.budget.limit)} budget` : "";
		lines.push(
			[
				"",
				`${ICON[row.severity]}${code(row.name)}`,
				g?.latest ? bytes(g.latest.bytes) : "—",
				g?.base ? bytes(g.base.bytes) : "—",
				g?.pr ? bytes(g.pr.bytes) : "—",
				r && g?.pr ? `${change(r.bytes, g.pr.bytes)}${budget}` : "new",
				r && g?.pr ? `${g.pr.count} (${change(r.count, g.pr.count, count)})` : String(g?.pr?.count ?? "—"),
				"",
			].join(" | ").trim(),
		);
	}
	return lines;
}

function details(row: PackageRow, ref: "latest" | "base" | "pr"): Array<string> {
	const out = [`<details><summary>${ICON[row.severity]}${code(row.name)} details</summary>`, ""];

	out.push(`| platform | ${ref} | this PR | Δ |`, "|---|---:|---:|---|");
	for (const p of row.install) {
		const r = p[ref];
		out.push(`| ${p.platform} | ${r ? bytes(r.bytes) : "—"} | ${p.pr ? bytes(p.pr.bytes) : "—"} | ${r && p.pr ? change(r.bytes, p.pr.bytes) : "—"} |`);
	}
	out.push("");

	const refs = (title: string, list: PackageRow["added"]) => {
		if (list.length === 0) return;
		out.push(`**${title}**`, "", "| package | size | via |", "|---|---:|---|");
		for (const p of list) out.push(`| ${code(p.id)} | ${bytes(p.bytes)} | ${code(p.via.join(" › "))} |`);
		out.push("");
	};
	refs("Added dependencies", row.added);
	refs("Removed dependencies", row.removed);
	refs("Largest dependencies", row.largest);

	if (row.installScriptsAdded.length > 0) {
		out.push(`**New install scripts:** ${row.installScriptsAdded.map(code).join(", ")}`, "");
	}
	if (row.floating.length > 0) {
		out.push("**Floating `0.x` ranges** (resolve above their floor today)", "", "| range | declared by | resolves to | subtree |", "|---|---|---|---:|");
		for (const f of row.floating) {
			out.push(`| ${code(`${f.dependency}@${f.range}`)} | ${code(f.declaredBy)} | ${code(f.resolved)} | ${bytes(f.subtreeBytes)} |`);
		}
		out.push("");
	}
	const entries = row.entrypoints.filter((e) => e.severity !== "ok" || hasSurfaceChange(e));
	if (entries.length > 0) {
		out.push("**Entrypoints** (own code, minified; third-party imports excluded)", "", "| entrypoint | base | this PR | Δ | imports | exports |", "|---|---:|---:|---|---|---|");
		for (const e of entries) {
			out.push(
				`| ${ICON[e.severity]}${code(e.subpath)} | ${e.base === undefined ? "—" : bytes(e.base)} | ${e.pr === undefined ? "—" : bytes(e.pr)} | ${e.base !== undefined && e.pr !== undefined ? change(e.base, e.pr) : "new"} | ${diffList(e.externalsAdded, e.externalsRemoved)} | ${diffList(e.exportsAdded, e.exportsRemoved)} |`,
			);
		}
		out.push("");
	}
	out.push("</details>", "");
	return out;
}

const hasSurfaceChange = (e: EntrypointRow) =>
	e.externalsAdded.length + e.externalsRemoved.length + e.exportsAdded.length + e.exportsRemoved.length > 0;

function diffList(added: ReadonlyArray<string>, removed: ReadonlyArray<string>): string {
	const parts = [...added.map((a) => `+${code(a)}`), ...removed.map((r) => `−${code(r)}`)];
	return parts.length === 0 ? "—" : parts.join(" ");
}
