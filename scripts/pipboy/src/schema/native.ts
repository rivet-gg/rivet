import { Schema } from "effect";

export const NativeArtifact = Schema.Struct({
	/** `rivetkit-napi` or `engine`. */
	target: Schema.String,
	platform: Schema.String,
	/** Size of the file we publish. */
	bytes: Schema.Number,
	/** Symbol bytes by crate, from the binary's symbol table. Empty when the binary has no symbols. */
	crates: Schema.Record(Schema.String, Schema.Number),
});
export interface NativeArtifact extends Schema.Schema.Type<typeof NativeArtifact> {}

export const NativeReport = Schema.Struct({
	schema: Schema.Literal(1),
	check: Schema.Literal("native"),
	artifact: NativeArtifact,
	/** Unpacked size of the published npm platform package, for reference. */
	latestBytes: Schema.optionalKey(Schema.Number),
});
export interface NativeReport extends Schema.Schema.Type<typeof NativeReport> {}
