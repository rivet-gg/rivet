import { builtinModules } from "node:module";
import { dirname, join, resolve } from "node:path";
import { Effect, FileSystem, Schema } from "effect";
import * as esbuild from "esbuild";
import ts from "typescript";
import type { Entrypoint } from "../schema/footprint.ts";
import { readPackManifest } from "./pack.ts";
import { readTarball } from "./tarball.ts";

export class InspectFailed extends Schema.TaggedError<InspectFailed>()("InspectFailed", {
	packageName: Schema.String,
	subpath: Schema.String,
	message: Schema.String,
}) {}

type Exports = string | null | { readonly [key: string]: Exports } | ReadonlyArray<Exports>;

const BUILTINS = new Set(builtinModules.flatMap((name) => [name, `node:${name}`]));

/**
 * Measures every entrypoint of our packed packages: minified own-code bytes,
 * third-party imports, and exported names. Our packages are inlined into each
 * other's bundles; third-party packages stay external, so no install is needed.
 */
export const inspectEntrypoints = Effect.fn("Footprint.inspectEntrypoints")(function* (tarballDir: string) {
	const fs = yield* FileSystem.FileSystem;
	const manifestDir = resolve(tarballDir);
	const manifest = yield* readPackManifest(manifestDir);

	const root = yield* fs.makeTempDirectoryScoped({ prefix: "pipboy-inspect-" });
	const nodeModules = join(root, "node_modules");
	const files = yield* Effect.forEach(
		Object.entries(manifest.tarballs),
		([name, file]) =>
			fs.readFile(join(manifestDir, file)).pipe(
				Effect.map((tarball) =>
					readTarball(tarball).map((entry) => ({
						path: join(nodeModules, name, entry.path.replace(/^package\//, "")),
						data: entry.data,
					})),
				),
			),
		{ concurrency: 8 },
	).pipe(Effect.map((perPackage) => perPackage.flat()));
	yield* Effect.forEach(new Set(files.map((f) => dirname(f.path))), (dir) => fs.makeDirectory(dir, { recursive: true }), {
		concurrency: 32,
		discard: true,
	});
	yield* Effect.forEach(files, (f) => fs.writeFile(f.path, f.data), { concurrency: 32, discard: true });

	const own = new Set(Object.keys(manifest.tarballs));
	const entries = yield* Effect.forEach(
		own,
		(name) =>
			Effect.gen(function* () {
				const pkgDir = join(nodeModules, name);
				const pkg = JSON.parse(yield* fs.readFileString(join(pkgDir, "package.json")));
				return entrypointTargets(pkg).flatMap(([subpath, target]) => {
					const js = pickCondition(target, ["import", "node", "default", "require"], /\.(m|c)?js$/);
					if (js === undefined) return [];
					const types = pickCondition(target, ["types", "import", "default"], /\.d\.(m|c)?ts$/);
					return [{ name, subpath, js: join(pkgDir, js), types: types && join(pkgDir, types) }];
				});
			}),
		{ concurrency: 8 },
	).pipe(Effect.map((perPackage) => perPackage.flat()));

	// esbuild and the TypeScript checker are CPU-bound; a few at a time keeps memory bounded.
	const measured = yield* Effect.forEach(
		entries,
		(e) => inspectEntry(e.name, e.subpath, e.js, e.types, own, nodeModules).pipe(Effect.map((m) => ({ ...e, m }))),
		{ concurrency: 4 },
	);
	const result: Record<string, Record<string, Entrypoint>> = Object.fromEntries([...own].map((name) => [name, {}]));
	for (const { name, subpath, m } of measured) (result[name] ??= {})[subpath] = m;
	return result;
});

const inspectEntry = (
	packageName: string,
	subpath: string,
	jsFile: string,
	typesFile: string | undefined,
	own: ReadonlySet<string>,
	nodeModules: string,
) =>
	Effect.tryPromise({
		try: async () => {
			const externals = new Set<string>();
			const built = await esbuild.build({
				entryPoints: [jsFile],
				bundle: true,
				minify: true,
				platform: "node",
				format: "esm",
				write: false,
				logLevel: "silent",
				nodePaths: [nodeModules],
				loader: { ".node": "empty", ".wasm": "empty" },
				plugins: [
					{
						name: "external-third-party",
						setup(build) {
							build.onResolve({ filter: /^[^./]/ }, (args) => {
								if (BUILTINS.has(args.path)) return { external: true };
								const name = bareName(args.path);
								if (own.has(name)) return undefined;
								externals.add(name);
								return { external: true };
							});
						},
					},
				],
			});
			return {
				bytes: built.outputFiles.reduce((sum, file) => sum + file.contents.byteLength, 0),
				externals: [...externals].sort(),
				exports: typesFile === undefined ? [] : exportedNames(typesFile),
			} satisfies Entrypoint;
		},
		catch: (cause) => new InspectFailed({ packageName, subpath, message: String(cause) }),
	});

function exportedNames(typesFile: string): Array<string> {
	const program = ts.createProgram([typesFile], {
		noEmit: true,
		skipLibCheck: true,
		types: [],
		module: ts.ModuleKind.ESNext,
		moduleResolution: ts.ModuleResolutionKind.Bundler,
	});
	const source = program.getSourceFile(typesFile);
	const checker = program.getTypeChecker();
	const symbol = source && checker.getSymbolAtLocation(source);
	if (!symbol) return [];
	return checker
		.getExportsOfModule(symbol)
		.map((s) => s.getName())
		.sort();
}

/** `[subpath, target]` pairs from `exports`, or the legacy `main`/`types` fields. */
function entrypointTargets(pkg: { exports?: Exports; main?: string; module?: string; types?: string }) {
	const exports = pkg.exports;
	if (exports === undefined || exports === null) {
		const main = pkg.module ?? pkg.main;
		return main ? ([[".", { import: main, types: pkg.types ?? null }]] as Array<[string, Exports]>) : [];
	}
	if (typeof exports === "string" || Array.isArray(exports)) return [[".", exports]] as Array<[string, Exports]>;
	const entries = Object.entries(exports as Record<string, Exports>);
	if (entries.every(([key]) => !key.startsWith("."))) return [[".", exports]] as Array<[string, Exports]>;
	return entries.filter(([key]) => key.startsWith(".") && !key.includes("*") && key !== "./package.json");
}

/** Walks conditional exports in condition order and returns the first path matching `pattern`. */
function pickCondition(target: Exports, conditions: ReadonlyArray<string>, pattern: RegExp): string | undefined {
	if (typeof target === "string") return pattern.test(target) ? target : undefined;
	if (target === null) return undefined;
	if (Array.isArray(target)) {
		for (const item of target) {
			const found = pickCondition(item, conditions, pattern);
			if (found !== undefined) return found;
		}
		return undefined;
	}
	const record = target as Record<string, Exports>;
	for (const condition of conditions) {
		if (condition in record) {
			const found = pickCondition(record[condition] ?? null, conditions, pattern);
			if (found !== undefined) return found;
		}
	}
	return undefined;
}

function bareName(specifier: string): string {
	const parts = specifier.split("/");
	return specifier.startsWith("@") ? `${parts[0]}/${parts[1]}` : (parts[0] ?? specifier);
}
