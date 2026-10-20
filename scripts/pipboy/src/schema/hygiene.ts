import { Schema } from "effect";

export const HygieneCheck = Schema.Literals(["advisories", "licenses", "sources", "bans", "unused"]);
export type HygieneCheck = Schema.Schema.Type<typeof HygieneCheck>;

export const HygieneFinding = Schema.Struct({
	check: HygieneCheck,
	/** Stable identity across commits, such as `vulnerability:RUSTSEC-2026-0007:bytes@1.10.1`. */
	key: Schema.String,
	title: Schema.String,
	/** Crate names from a workspace crate down to the offending one, or the manifest path for unused deps. */
	via: Schema.String,
});
export interface HygieneFinding extends Schema.Schema.Type<typeof HygieneFinding> {}

export const HygieneSide = Schema.Struct({
	side: Schema.Literals(["base", "pr"]),
	findings: Schema.Array(HygieneFinding),
});
export interface HygieneSide extends Schema.Schema.Type<typeof HygieneSide> {}

export const HygieneReport = Schema.Struct({
	schema: Schema.Literal(1),
	check: Schema.Literal("hygiene"),
	sides: Schema.Array(HygieneSide),
});
export interface HygieneReport extends Schema.Schema.Type<typeof HygieneReport> {}
