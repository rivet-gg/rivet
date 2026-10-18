import { join } from "node:path";
import { Effect, FileSystem, Schema } from "effect";
import { discoverPackages } from "publish/src/lib/packages.ts";
import { PACK_VERSION, REGISTRY_PINNED, REPO_ROOT } from "../config.ts";
import { run } from "../exec.ts";
import { PackManifest } from "../schema/footprint.ts";

export class PackFailed extends Schema.TaggedError<PackFailed>()("PackFailed", {
	packageName: Schema.String,
	expected: Schema.String,
}) {
	override get message() {
		return `pnpm pack did not produce ${this.expected} for ${this.packageName}`;
	}
}

/** Workspace packages `npm publish` ships, minus per-platform binaries and registry-pinned metas. */
export const packablePackages = (root: string = REPO_ROOT) =>
	discoverPackages(root).filter((pkg) => !pkg.relDir.includes("/npm/") && !REGISTRY_PINNED.includes(pkg.name));

/**
 * Builds and packs every publishable workspace package of the checkout at
 * `root` into `outDir`, the way `npm publish` would. All packages get
 * `PACK_VERSION` so `workspace:*` references between them resolve to these
 * tarballs. Original `package.json` files are restored afterwards, even on
 * failure. Native builds are skipped: those packages come from the registry.
 */
export const pack = Effect.fn("Footprint.pack")(function* (root: string, outDir: string) {
	const fs = yield* FileSystem.FileSystem;
	yield* fs.makeDirectory(outDir, { recursive: true });
	const packages = packablePackages(root);

	yield* run("pnpm", ["exec", "turbo", "run", "build", "--output-logs=errors-only", ...packages.map((pkg) => `--filter=${pkg.name}`)], {
		cwd: root,
		env: { SKIP_NAPI_BUILD: "1", SKIP_WASM_BUILD: "1" },
	});

	const originals = yield* Effect.forEach(packages, (pkg) =>
		fs.readFileString(join(pkg.dir, "package.json")).pipe(Effect.map((text) => ({ pkg, text }))),
	);
	const restore = Effect.forEach(
		originals,
		({ pkg, text }) => fs.writeFileString(join(pkg.dir, "package.json"), text),
		{ discard: true },
	).pipe(Effect.orDie);

	const tarballs = yield* Effect.acquireUseRelease(
		Effect.forEach(
			originals,
			({ pkg, text }) =>
				fs.writeFileString(
					join(pkg.dir, "package.json"),
					`${JSON.stringify({ ...JSON.parse(text), version: PACK_VERSION }, null, "\t")}\n`,
				),
			{ discard: true },
		),
		() =>
			Effect.forEach(
				packages,
				(pkg) =>
					Effect.gen(function* () {
						yield* run("pnpm", ["pack", "--pack-destination", outDir], { cwd: pkg.dir });
						// pnpm names tarballs `<scope>-<name>-<version>.tgz`. Prepack scripts can
						// print to stdout, so the name is derived rather than parsed.
						const file = `${pkg.name.replace(/^@/, "").replace("/", "-")}-${PACK_VERSION}.tgz`;
						if (!(yield* fs.exists(join(outDir, file)))) {
							return yield* new PackFailed({ packageName: pkg.name, expected: file });
						}
						return [pkg.name, file] as const;
					}),
				{ concurrency: 4 },
			),
		() => restore,
	);

	const manifest: PackManifest = { version: PACK_VERSION, tarballs: Object.fromEntries(tarballs) };
	yield* fs.writeFileString(join(outDir, "manifest.json"), JSON.stringify(manifest, null, 2));
	return manifest;
});

/** The manifest `pack` writes next to the tarballs. */
export const readPackManifest = Effect.fn("Footprint.readPackManifest")(function* (dir: string) {
	const fs = yield* FileSystem.FileSystem;
	return yield* Schema.decodeUnknownEffect(Schema.fromJsonString(PackManifest))(yield* fs.readFileString(join(dir, "manifest.json")));
});
