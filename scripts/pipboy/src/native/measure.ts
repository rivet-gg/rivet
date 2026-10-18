import { Effect, FileSystem, Option } from "effect";
import { capture } from "../exec.ts";
import { Registry } from "../footprint/registry.ts";
import type { NativeReport } from "../schema/native.ts";

/** The npm package that ships each build artifact. */
const NPM_PACKAGE: Record<string, (platform: string) => string> = {
	"rivetkit-napi": (platform) => `@rivetkit/rivetkit-napi-${platform.startsWith("windows") ? "win32-x64-msvc" : platform}`,
	engine: (platform) => `@rivetkit/engine-cli-${platform.startsWith("windows") ? "win32-x64" : platform}`,
};

/**
 * Measures one published binary: its size, and symbol bytes per crate. Crates
 * come from demangled symbol names (`crate::module::item`), so the split is
 * approximate; symbols without a Rust path count as `[c/other]`.
 */
export const measureNative = Effect.fn("Native.measure")(function* (target: string, platform: string, path: string) {
	const fs = yield* FileSystem.FileSystem;
	const bytes = Number((yield* fs.stat(path)).size);
	const crates = yield* crateSizes(path);

	const latestBytes = yield* latestPublishedSize(NPM_PACKAGE[target]?.(platform));
	return {
		schema: 1,
		check: "native",
		artifact: { target, platform, bytes, crates },
		...(latestBytes !== undefined ? { latestBytes } : {}),
	} satisfies NativeReport;
});

/** Unpacked size of the npm package's `latest` version, if it's published. */
const latestPublishedSize = Effect.fn("Native.latestPublishedSize")(function* (name: string | undefined) {
	if (name === undefined) return undefined;
	const registry = yield* Registry;
	const version = yield* registry.latestVersion(name);
	return Option.isSome(version) ? yield* registry.packageSize(name, version.value) : undefined;
});

// Stripped binaries (the musl Dockerfiles strip `rivet-engine`) and Mach-O or PE binaries
// the runner's GNU nm can't read report no crates.
const crateSizes = (path: string) =>
	capture("nm", ["--demangle", "--print-size", "--size-sort", path]).pipe(
		Effect.map(({ exitCode, stdout }) => {
			const crates: Record<string, number> = {};
			if (exitCode !== 0) return crates;
			for (const line of stdout.split("\n")) {
				// `<address> <size> <type> <symbol>`, sizes in hex.
				const match = line.match(/^[0-9a-f]+ ([0-9a-f]+) \S (.+)$/i);
				if (!match) continue;
				const crate = match[2]?.match(/([A-Za-z_][A-Za-z0-9_]*)::/)?.[1] ?? "[c/other]";
				crates[crate] = (crates[crate] ?? 0) + Number.parseInt(match[1] ?? "0", 16);
			}
			return crates;
		}),
	);
