import { Schema } from "effect";

/** One resolved package in a side's dependency graph, keyed by its lockfile path. */
export const PackageNode = Schema.Struct({
	name: Schema.String,
	version: Schema.String,
	/** Unpacked size in bytes. */
	bytes: Schema.Number,
	os: Schema.optionalKey(Schema.Array(Schema.String)),
	cpu: Schema.optionalKey(Schema.Array(Schema.String)),
	libc: Schema.optionalKey(Schema.Array(Schema.String)),
	installScript: Schema.Boolean,
	/** Lockfile keys of the packages this one depends on. */
	deps: Schema.Array(Schema.String),
	/** Declared `^0.x` ranges, by dependency name. */
	zeroXRanges: Schema.Record(Schema.String, Schema.String),
});
export interface PackageNode extends Schema.Schema.Type<typeof PackageNode> {}

/** Measurements of one entrypoint of a package we publish. */
export const Entrypoint = Schema.Struct({
	/** Minified bytes of our own code, third-party imports excluded. */
	bytes: Schema.Number,
	/** Third-party packages the entrypoint imports. */
	externals: Schema.Array(Schema.String),
	/** Exported names from the entrypoint's type declarations. */
	exports: Schema.Array(Schema.String),
});
export interface Entrypoint extends Schema.Schema.Type<typeof Entrypoint> {}

export const SideName = Schema.Literals(["latest", "base", "pr"]);
export type SideName = Schema.Schema.Type<typeof SideName>;

export const FootprintSide = Schema.Struct({
	side: SideName,
	npmVersion: Schema.String,
	/** Root package name to its lockfile key. */
	roots: Schema.Record(Schema.String, Schema.String),
	nodes: Schema.Record(Schema.String, PackageNode),
	/** Our packages' entrypoints by package name, then export subpath. Empty for `latest`. */
	entrypoints: Schema.Record(Schema.String, Schema.Record(Schema.String, Entrypoint)),
});
export interface FootprintSide extends Schema.Schema.Type<typeof FootprintSide> {}

export const FootprintReport = Schema.Struct({
	schema: Schema.Literal(1),
	check: Schema.Literal("footprint"),
	sides: Schema.Array(FootprintSide),
});
export interface FootprintReport extends Schema.Schema.Type<typeof FootprintReport> {}

/** Written by `pack`, read by `footprint`. */
export const PackManifest = Schema.Struct({
	version: Schema.String,
	/** Package name to tarball file name, relative to the manifest. */
	tarballs: Schema.Record(Schema.String, Schema.String),
});
export interface PackManifest extends Schema.Schema.Type<typeof PackManifest> {}
