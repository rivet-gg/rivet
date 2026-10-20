import { BUDGETS, GATED_PLATFORM, PLATFORMS, type Platform, THRESHOLDS, type Threshold } from "../config.ts";
import type { FootprintReport, FootprintSide, PackageNode, SideName } from "../schema/footprint.ts";

export type Severity = "ok" | "warn" | "alert" | "fail";

export interface Measure {
	readonly bytes: number;
	readonly count: number;
}

export interface PackageRef {
	readonly id: string;
	readonly bytes: number;
	/** Package names from the root down to this package. */
	readonly via: ReadonlyArray<string>;
}

export interface FloatingRange {
	readonly declaredBy: string;
	readonly dependency: string;
	readonly range: string;
	readonly resolved: string;
	/** Install size of the dependency's subtree on the gated platform. */
	readonly subtreeBytes: number;
}

export interface EntrypointRow {
	readonly subpath: string;
	readonly base?: number;
	readonly pr?: number;
	readonly severity: Severity;
	readonly externalsAdded: ReadonlyArray<string>;
	readonly externalsRemoved: ReadonlyArray<string>;
	readonly exportsAdded: ReadonlyArray<string>;
	readonly exportsRemoved: ReadonlyArray<string>;
}

export interface PackageRow {
	readonly name: string;
	readonly severity: Severity;
	/** Install size per platform and side. */
	readonly install: ReadonlyArray<{ readonly platform: string } & Partial<Record<SideName, Measure>>>;
	readonly budget?: { readonly limit: number; readonly over: Partial<Record<SideName, boolean>> };
	readonly added: ReadonlyArray<PackageRef>;
	readonly removed: ReadonlyArray<PackageRef>;
	readonly installScriptsAdded: ReadonlyArray<string>;
	readonly largest: ReadonlyArray<PackageRef>;
	readonly floating: ReadonlyArray<FloatingRange>;
	readonly entrypoints: ReadonlyArray<EntrypointRow>;
}

export interface FootprintComparison {
	/** The side the PR is compared against: `base` on PRs, `latest` on main pushes. */
	readonly reference: SideName;
	readonly rows: ReadonlyArray<PackageRow>;
	/** Packages whose `latest` already exceeds its budget: drift no PR caused. */
	readonly latestOverBudget: ReadonlyArray<string>;
	readonly failures: ReadonlyArray<string>;
}

const TOP = 10;

export function compareFootprint(report: FootprintReport): FootprintComparison {
	const sides = new Map(report.sides.map((side) => [side.side, side]));
	const pr = sides.get("pr");
	if (!pr) throw new Error("footprint report has no pr side");
	const reference: SideName = sides.has("base") ? "base" : "latest";
	const ref = sides.get(reference);

	const own = new Set(Object.keys(pr.roots));
	const rows = Object.keys(pr.roots)
		.sort()
		.map((name) => compareRoot(name, sides, pr, ref, own));
	const failures = rows.flatMap((row) => {
		if (row.severity !== "fail" || !row.budget) return [];
		const bytes = row.install[0]?.pr?.bytes ?? 0;
		return [`${row.name} installs ${bytes} bytes on ${GATED_PLATFORM.id}, over its ${row.budget.limit} byte budget`];
	});
	return {
		reference,
		rows,
		latestOverBudget: rows.filter((row) => row.budget?.over.latest === true).map((row) => row.name),
		failures,
	};
}

function compareRoot(
	name: string,
	sides: ReadonlyMap<SideName, FootprintSide>,
	pr: FootprintSide,
	ref: FootprintSide | undefined,
	own: ReadonlySet<string>,
): PackageRow {
	const install = PLATFORMS.map((platform) => {
		const measures: Partial<Record<SideName, Measure>> = {};
		for (const [sideName, side] of sides) {
			const key = side.roots[name];
			if (key !== undefined) measures[sideName] = measure(side, closure(side, key, platform));
		}
		return { platform: platform.id, ...measures };
	});
	const gated = install[0] ?? { platform: GATED_PLATFORM.id };

	const limit = BUDGETS[name];
	const over: Partial<Record<SideName, boolean>> = {};
	if (limit !== undefined) {
		for (const sideName of sides.keys()) {
			const bytes = gated[sideName]?.bytes;
			if (bytes !== undefined) over[sideName] = bytes > limit;
		}
	}

	const prClosure = closure(pr, pr.roots[name] ?? "", GATED_PLATFORM);
	const refKey = ref?.roots[name];
	const refClosure = ref && refKey !== undefined ? closure(ref, refKey, GATED_PLATFORM) : new Map<string, Array<string>>();
	const prIds = byId(pr, prClosure, own);
	const refIds = ref ? byId(ref, refClosure, own) : new Map<string, PackageRef>();

	const added = [...prIds.values()].filter((p) => !refIds.has(p.id)).sort(byBytes);
	const removed = [...refIds.values()].filter((p) => !prIds.has(p.id)).sort(byBytes);
	const scripts = (side: FootprintSide, c: Map<string, Array<string>>) =>
		new Set([...c.keys()].flatMap((key) => (side.nodes[key]?.installScript ? [nodeId(side.nodes[key])] : [])));
	const refScripts = ref ? scripts(ref, refClosure) : new Set<string>();
	const installScriptsAdded = [...scripts(pr, prClosure)].filter((id) => !refScripts.has(id)).sort();

	const entrypoints = compareEntrypoints(name, ref, pr);
	const reference = ref?.side;
	let severity = worst([
		reference ? level(gated[reference]?.bytes, gated.pr?.bytes, THRESHOLDS.install) : "ok",
		reference ? level(gated[reference]?.count, gated.pr?.count, THRESHOLDS.packages) : "ok",
		installScriptsAdded.length > 0 ? "alert" : "ok",
		...entrypoints.map((e) => e.severity),
	]);
	// Fail only when the PR itself pushes the package over budget, not when the
	// reference already was (registry drift).
	if (over.pr === true && (reference === undefined || over[reference] !== true)) severity = "fail";

	const flagged = severity !== "ok" || over.pr === true;
	return {
		name,
		severity,
		install,
		...(limit !== undefined ? { budget: { limit, over } } : {}),
		added: added.slice(0, TOP),
		removed: removed.slice(0, TOP),
		installScriptsAdded,
		largest: flagged ? [...prIds.values()].filter((p) => p.via.length > 1).sort(byBytes).slice(0, 5) : [],
		floating: flagged ? floatingRanges(pr, prClosure) : [],
		entrypoints,
	};
}

/** Every package installed with `rootKey` on `platform`, mapped to the names on its path from the root. */
export function closure(side: FootprintSide, rootKey: string, platform: Platform): Map<string, Array<string>> {
	const seen = new Map<string, Array<string>>();
	const root = side.nodes[rootKey];
	if (!root || !installs(root, platform)) return seen;
	seen.set(rootKey, [root.name]);
	const queue = [rootKey];
	while (queue.length > 0) {
		const key = queue.shift() ?? "";
		const via = seen.get(key) ?? [];
		for (const dep of side.nodes[key]?.deps ?? []) {
			const node = side.nodes[dep];
			if (seen.has(dep) || !node || !installs(node, platform)) continue;
			seen.set(dep, [...via, node.name]);
			queue.push(dep);
		}
	}
	return seen;
}

/** npm's `os`/`cpu`/`libc` check. `libc` is only satisfiable on Linux. */
function installs(node: PackageNode, platform: Platform): boolean {
	return (
		matches(node.os, platform.os) &&
		matches(node.cpu, platform.cpu) &&
		(node.libc === undefined || (platform.libc !== undefined && matches(node.libc, platform.libc)))
	);
}

function matches(list: ReadonlyArray<string> | undefined, value: string): boolean {
	if (!list || list.length === 0) return true;
	if (list.includes(`!${value}`)) return false;
	const allowed = list.filter((item) => !item.startsWith("!"));
	return allowed.length === 0 || allowed.includes(value);
}

function measure(side: FootprintSide, packages: Map<string, Array<string>>): Measure {
	return { bytes: sumBytes(side, packages), count: packages.size };
}

/**
 * Packages by `name@version`. Our own packages are keyed by name alone: the PR
 * side packs them under a synthetic version, so a version change isn't news.
 */
function byId(side: FootprintSide, packages: Map<string, Array<string>>, own: ReadonlySet<string>): Map<string, PackageRef> {
	const out = new Map<string, PackageRef>();
	for (const [key, via] of packages) {
		const node = side.nodes[key];
		if (!node) continue;
		const id = own.has(node.name) ? node.name : nodeId(node);
		if (!out.has(id)) out.set(id, { id, bytes: node.bytes, via });
	}
	return out;
}

function floatingRanges(side: FootprintSide, packages: Map<string, Array<string>>): Array<FloatingRange> {
	const out: Array<FloatingRange> = [];
	for (const key of packages.keys()) {
		const node = side.nodes[key];
		if (!node) continue;
		for (const [dependency, range] of Object.entries(node.zeroXRanges)) {
			const depKey = node.deps.find((d) => side.nodes[d]?.name === dependency);
			const dep = depKey === undefined ? undefined : side.nodes[depKey];
			if (!dep || depKey === undefined || dep.version === range.slice(1)) continue;
			out.push({
				declaredBy: nodeId(node),
				dependency,
				range,
				resolved: dep.version,
				subtreeBytes: sumBytes(side, closure(side, depKey, GATED_PLATFORM)),
			});
		}
	}
	return out.sort((a, b) => b.subtreeBytes - a.subtreeBytes).slice(0, 5);
}

/** Entrypoints are only measured for packed sides, so `latest` has nothing to compare against. */
function compareEntrypoints(name: string, ref: FootprintSide | undefined, pr: FootprintSide): Array<EntrypointRow> {
	if (ref?.side !== "base") return [];
	const before = ref.entrypoints[name] ?? {};
	const after = pr.entrypoints[name] ?? {};
	return [...new Set([...Object.keys(before), ...Object.keys(after)])].sort().map((subpath) => {
		const b = before[subpath];
		const a = after[subpath];
		const diff = (x: ReadonlyArray<string> = [], y: ReadonlyArray<string> = []) => y.filter((v) => !x.includes(v));
		const exportsRemoved = b && a ? diff(a.exports, b.exports) : [];
		return {
			subpath,
			...(b ? { base: b.bytes } : {}),
			...(a ? { pr: a.bytes } : {}),
			severity: worst([level(b?.bytes, a?.bytes, THRESHOLDS.bundle), exportsRemoved.length > 0 ? "alert" : "ok"]),
			externalsAdded: b && a ? diff(b.externals, a.externals) : [],
			externalsRemoved: b && a ? diff(a.externals, b.externals) : [],
			exportsAdded: b && a ? diff(b.exports, a.exports) : [],
			exportsRemoved,
		};
	});
}

/** Flags a growth that crosses both the relative and the absolute threshold. Shrinking is never flagged. */
export function level(
	before: number | undefined,
	after: number | undefined,
	t: { warn: Threshold; alert: Threshold },
): Severity {
	if (before === undefined || after === undefined || after <= before) return "ok";
	const delta = after - before;
	const pct = before === 0 ? Number.POSITIVE_INFINITY : (delta / before) * 100;
	const crosses = (x: Threshold) => pct >= x.pct && delta >= x.abs;
	return crosses(t.alert) ? "alert" : crosses(t.warn) ? "warn" : "ok";
}

const ORDER: ReadonlyArray<Severity> = ["ok", "warn", "alert", "fail"];
function worst(levels: ReadonlyArray<Severity>): Severity {
	return levels.reduce<Severity>((a, b) => (ORDER.indexOf(b) > ORDER.indexOf(a) ? b : a), "ok");
}

const nodeId = (node: PackageNode) => `${node.name}@${node.version}`;
const byBytes = (a: PackageRef, b: PackageRef) => b.bytes - a.bytes;
const sumBytes = (side: FootprintSide, packages: Map<string, Array<string>>) =>
	[...packages.keys()].reduce((sum, key) => sum + (side.nodes[key]?.bytes ?? 0), 0);
