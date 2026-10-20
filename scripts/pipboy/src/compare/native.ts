import { THRESHOLDS } from "../config.ts";
import type { NativeReport } from "../schema/native.ts";
import { type Severity, level } from "./footprint.ts";

export interface NativeRow {
	readonly target: string;
	readonly platform: string;
	readonly base?: number;
	readonly pr: number;
	readonly latest?: number;
	readonly severity: Severity;
	/** Crates whose symbol bytes changed most, largest change first. */
	readonly crates: ReadonlyArray<{ readonly name: string; readonly base: number; readonly pr: number }>;
}

/** Pairs each PR artifact with its baseline from main, if one was found. */
export function compareNative(
	pr: ReadonlyArray<NativeReport>,
	base: ReadonlyArray<NativeReport>,
): ReadonlyArray<NativeRow> {
	return pr
		.map((report) => {
			const a = report.artifact;
			const b = base.find((r) => r.artifact.target === a.target && r.artifact.platform === a.platform)?.artifact;
			const names = new Set([...Object.keys(a.crates), ...Object.keys(b?.crates ?? {})]);
			const crates = [...names]
				.map((name) => ({ name, base: b?.crates[name] ?? 0, pr: a.crates[name] ?? 0 }))
				.filter((c) => c.base !== c.pr)
				.sort((x, y) => Math.abs(y.pr - y.base) - Math.abs(x.pr - x.base))
				.slice(0, 10);
			return {
				target: a.target,
				platform: a.platform,
				...(b ? { base: b.bytes } : {}),
				pr: a.bytes,
				...(report.latestBytes !== undefined ? { latest: report.latestBytes } : {}),
				severity: level(b?.bytes, a.bytes, THRESHOLDS.binary),
				crates: b ? crates : [],
			};
		})
		.sort((x, y) => `${x.target}${x.platform}`.localeCompare(`${y.target}${y.platform}`));
}
