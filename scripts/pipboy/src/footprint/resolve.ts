import { join, resolve } from "node:path";
import { Effect, FileSystem, Option, Schema } from "effect";
import { REGISTRY_PINNED } from "../config.ts";
import { npmEnv, run } from "../exec.ts";
import { type PackageNode, type SideName } from "../schema/footprint.ts";
import { readPackManifest } from "./pack.ts";
import { Registry } from "./registry.ts";
import { unpackedSize } from "./tarball.ts";

const StringRecord = Schema.Record(Schema.String, Schema.String);

const LockEntry = Schema.Struct({
	name: Schema.optionalKey(Schema.String),
	version: Schema.optionalKey(Schema.String),
	resolved: Schema.optionalKey(Schema.String),
	dependencies: Schema.optionalKey(StringRecord),
	optionalDependencies: Schema.optionalKey(StringRecord),
	peerDependencies: Schema.optionalKey(StringRecord),
	peerDependenciesMeta: Schema.optionalKey(
		Schema.Record(Schema.String, Schema.Struct({ optional: Schema.optionalKey(Schema.Boolean) })),
	),
	os: Schema.optionalKey(Schema.Array(Schema.String)),
	cpu: Schema.optionalKey(Schema.Array(Schema.String)),
	libc: Schema.optionalKey(Schema.Array(Schema.String)),
	hasInstallScript: Schema.optionalKey(Schema.Boolean),
});
type LockEntry = Schema.Schema.Type<typeof LockEntry>;

const Lockfile = Schema.Struct({
	lockfileVersion: Schema.Literal(3),
	packages: Schema.Record(Schema.String, LockEntry),
});

export type SideInput =
	| { readonly side: "latest"; readonly roots: ReadonlyArray<string> }
	| { readonly side: Exclude<SideName, "latest">; readonly tarballDir: string };

/**
 * Resolves a side's dependency tree exactly as `npm install` would, without
 * downloading it: `npm install --package-lock-only` writes the lockfile, and
 * sizes come from registry metadata. Install scripts never run.
 */
export const resolveSide = Effect.fn("Footprint.resolveSide")(function* (input: SideInput) {
	const fs = yield* FileSystem.FileSystem;
	const registry = yield* Registry;
	const dir = yield* fs.makeTempDirectoryScoped({ prefix: `pipboy-${input.side}-` });

	/** `name -> latest version` for the packages in `names` that are published. */
	const latestVersions = (names: ReadonlyArray<string>) =>
		Effect.forEach(names, (name) => registry.latestVersion(name).pipe(Effect.map((v) => [name, v] as const)), {
			concurrency: 16,
		}).pipe(
			Effect.map((pairs) => Object.fromEntries(pairs.flatMap(([name, v]) => (Option.isSome(v) ? [[name, v.value]] : [])))),
		);

	const pinned = yield* latestVersions(REGISTRY_PINNED);
	const localTarballs = new Map<string, string>();
	let dependencies: Record<string, string>;
	if (input.side === "latest") {
		dependencies = yield* latestVersions(input.roots);
	} else {
		const manifestDir = resolve(input.tarballDir);
		const manifest = yield* readPackManifest(manifestDir);
		dependencies = {};
		for (const [name, file] of Object.entries(manifest.tarballs)) {
			const path = join(manifestDir, file);
			dependencies[name] = `file:${path}`;
			localTarballs.set(name, path);
		}
	}

	yield* fs.writeFileString(
		join(dir, "package.json"),
		JSON.stringify({ name: `pipboy-${input.side}`, private: true, dependencies, overrides: pinned }),
	);
	yield* run("npm", ["install", "--package-lock-only", "--ignore-scripts", "--no-audit", "--no-fund"], { cwd: dir, env: npmEnv() });
	const npmVersion = (yield* run("npm", ["--version"], { env: npmEnv() })).trim();
	const lockfile = yield* fs
		.readFileString(join(dir, "package-lock.json"))
		.pipe(Effect.flatMap(Schema.decodeUnknownEffect(Schema.fromJsonString(Lockfile))));

	const entries = Object.entries(lockfile.packages).filter(([key]) => key !== "");
	const sizes = new Map(
		yield* Effect.forEach(
			entries,
			([key, entry]) =>
				Effect.gen(function* () {
					const local = localTarballs.get(packageName(key, entry));
					const bytes =
						local !== undefined
							? unpackedSize(yield* fs.readFile(local))
							: yield* registry.packageSize(packageName(key, entry), entry.version ?? "");
					return [key, bytes] as const;
				}),
			{ concurrency: 32 },
		),
	);

	const nodes: Record<string, PackageNode> = {};
	for (const [key, entry] of entries) {
		nodes[key] = {
			name: packageName(key, entry),
			version: entry.version ?? "",
			bytes: sizes.get(key) ?? 0,
			...(entry.os ? { os: entry.os } : {}),
			...(entry.cpu ? { cpu: entry.cpu } : {}),
			...(entry.libc ? { libc: entry.libc } : {}),
			installScript: entry.hasInstallScript ?? false,
			deps: installedDependencies(entry).flatMap((dep) => {
				const found = lookup(lockfile.packages, key, dep);
				return found === undefined ? [] : [found];
			}),
			zeroXRanges: Object.fromEntries(
				Object.entries({ ...entry.dependencies, ...entry.optionalDependencies }).filter(([, range]) =>
					/^\^0\./.test(range),
				),
			),
		};
	}

	const roots = Object.fromEntries(
		Object.keys(dependencies).flatMap((name) => {
			const key = lookup(lockfile.packages, "", name);
			return key === undefined ? [] : [[name, key] as const];
		}),
	);
	return { side: input.side, npmVersion, roots, nodes, entrypoints: {} };
});

function packageName(key: string, entry: LockEntry): string {
	return entry.name ?? key.slice(key.lastIndexOf("node_modules/") + "node_modules/".length);
}

/** Dependencies npm installs by default: regular, optional, and non-optional peers. */
function installedDependencies(entry: LockEntry): Array<string> {
	const peers = Object.keys(entry.peerDependencies ?? {}).filter(
		(name) => entry.peerDependenciesMeta?.[name]?.optional !== true,
	);
	return [...Object.keys(entry.dependencies ?? {}), ...Object.keys(entry.optionalDependencies ?? {}), ...peers];
}

/** Node's module lookup over lockfile keys: nearest `node_modules` first, then each ancestor. */
function lookup(packages: Record<string, unknown>, from: string, dep: string): string | undefined {
	let base = from;
	while (true) {
		const candidate = base === "" ? `node_modules/${dep}` : `${base}/node_modules/${dep}`;
		if (candidate in packages) return candidate;
		if (base === "") return undefined;
		const parent = base.lastIndexOf("/node_modules/");
		base = parent === -1 ? "" : base.slice(0, parent);
	}
}
