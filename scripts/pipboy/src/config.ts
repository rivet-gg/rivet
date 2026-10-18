import { resolve } from "node:path";

export const REPO_ROOT = resolve(import.meta.dirname, "../../..");

export interface Platform {
	readonly id: string;
	readonly os: string;
	readonly cpu: string;
	/** npm `libc` value. Only set for Linux. */
	readonly libc?: string;
}

/** Platforms whose install size is reported. The first one is gated by budgets. */
export const PLATFORMS: ReadonlyArray<Platform> = [
	{ id: "linux-x64-gnu", os: "linux", cpu: "x64", libc: "glibc" },
	{ id: "linux-arm64-gnu", os: "linux", cpu: "arm64", libc: "glibc" },
	{ id: "darwin-arm64", os: "darwin", cpu: "arm64" },
	{ id: "darwin-x64", os: "darwin", cpu: "x64" },
	{ id: "win32-x64", os: "win32", cpu: "x64" },
];
export const GATED_PLATFORM = PLATFORMS[0];

/**
 * Packages always resolved from the registry's `latest`, on every side. Their
 * publish step injects per-platform binaries that a workspace pack can't
 * reproduce. The `native` check measures those binaries.
 */
export const REGISTRY_PINNED = [
	"@rivetkit/rivetkit-napi",
	"@rivetkit/rivetkit-wasm",
	"@rivetkit/engine-cli",
	"@rivetkit/cli",
];

/**
 * Fresh install size caps on the gated platform, in bytes. `rivetkit` measured
 * 420 MB with a healthy dependency tree (2026-10-18), plus 20% headroom.
 */
export const BUDGETS: Record<string, number> = {
	rivetkit: 500 * 1024 ** 2,
};

export interface Threshold {
	readonly pct: number;
	readonly abs: number;
}

/**
 * A growth is flagged when it crosses both the relative and the absolute
 * threshold, so tiny packages don't alert on a few kilobytes.
 */
export const THRESHOLDS = {
	install: { warn: { pct: 5, abs: 1024 ** 2 }, alert: { pct: 20, abs: 10 * 1024 ** 2 } },
	packages: { warn: { pct: 10, abs: 5 }, alert: { pct: 25, abs: 25 } },
	bundle: { warn: { pct: 5, abs: 5 * 1024 }, alert: { pct: 20, abs: 25 * 1024 } },
} satisfies Record<string, { warn: Threshold; alert: Threshold }>;

/** Synthetic version for packed workspace packages, so npm can never pick a registry copy instead. */
export const PACK_VERSION = "0.0.0-pipboy";
