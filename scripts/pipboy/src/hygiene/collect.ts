import { Effect, Schema } from "effect";
import { type Captured, capture, commandFailed } from "../exec.ts";
import type { HygieneCheck, HygieneFinding } from "../schema/hygiene.ts";

interface Krate {
	readonly name: string;
	readonly version: string;
}
interface Graph {
	readonly Krate: Krate;
	readonly parents?: ReadonlyArray<Graph>;
}

const Graph: Schema.Codec<Graph> = Schema.Struct({
	Krate: Schema.Struct({ name: Schema.String, version: Schema.String }),
	parents: Schema.optionalKey(Schema.Array(Schema.suspend(() => Graph))),
});

const DenyLine = Schema.Struct({
	type: Schema.String,
	fields: Schema.optionalKey(
		Schema.Struct({
			severity: Schema.optionalKey(Schema.String),
			code: Schema.optionalKey(Schema.String),
			message: Schema.optionalKey(Schema.String),
			advisory: Schema.optionalKey(Schema.NullOr(Schema.Struct({ id: Schema.String }))),
			graphs: Schema.optionalKey(Schema.Array(Graph)),
		}),
	),
});

const ShearOutput = Schema.Struct({
	findings: Schema.Array(
		Schema.Struct({
			code: Schema.String,
			severity: Schema.String,
			message: Schema.String,
			file: Schema.optionalKey(Schema.String),
		}),
	),
});

/**
 * Every cargo-deny 0.20.2 diagnostic code, by check, from the `Code` enums in
 * its source. Update together with the pinned cargo-deny version.
 */
const DENY_CODES: Record<string, HygieneCheck> = {
	"vulnerability": "advisories",
	"notice": "advisories",
	"unmaintained": "advisories",
	"unsound": "advisories",
	"yanked": "advisories",
	"advisory-ignored": "advisories",
	"yanked-ignored": "advisories",
	"index-failure": "advisories",
	"index-cache-load-failure": "advisories",
	"advisory-not-detected": "advisories",
	"yanked-not-detected": "advisories",
	"unknown-advisory": "advisories",
	"accepted": "licenses",
	"rejected": "licenses",
	"unlicensed": "licenses",
	"skipped-private-workspace-crate": "licenses",
	"license-not-encountered": "licenses",
	"license-exception-not-encountered": "licenses",
	"missing-clarification-file": "licenses",
	"parse-error": "licenses",
	"empty-license-field": "licenses",
	"no-license-field": "licenses",
	"gather-failure": "licenses",
	"git-source-underspecified": "sources",
	"allowed-source": "sources",
	"allowed-by-organization": "sources",
	"source-not-allowed": "sources",
	"unmatched-source": "sources",
	"unmatched-organization": "sources",
	"banned": "bans",
	"allowed": "bans",
	"not-allowed": "bans",
	"duplicate": "bans",
	"skipped": "bans",
	"wildcard": "bans",
	"unmatched-skip": "bans",
	"unnecessary-skip": "bans",
	"allowed-by-wrapper": "bans",
	"unmatched-wrapper": "bans",
	"skipped-by-root": "bans",
	"unmatched-skip-root": "bans",
	"build-script-not-allowed": "bans",
	"exact-features-mismatch": "bans",
	"feature-not-explicitly-allowed": "bans",
	"feature-banned": "bans",
	"unknown-feature": "bans",
	"default-feature-enabled": "bans",
	"path-bypassed": "bans",
	"path-bypassed-by-glob": "bans",
	"checksum-match": "bans",
	"checksum-mismatch": "bans",
	"denied-by-extension": "bans",
	"detected-executable": "bans",
	"detected-executable-script": "bans",
	"unable-to-check-path": "bans",
	"features-enabled": "bans",
	"unmatched-bypass": "bans",
	"unmatched-path-bypass": "bans",
	"unmatched-glob": "bans",
	"unused-wrapper": "bans",
	"workspace-duplicate": "bans",
	"unresolved-workspace-dependency": "bans",
	"unused-workspace-dependency": "bans",
	"non-root-path": "bans",
	"replaced-in-std": "bans",
	"unmatched-replacement-ignore": "bans",
};

/** Names from a workspace crate down to the reported crate, following the first parent at each level. */
function via(graph: Graph): string {
	const names: Array<string> = [];
	let node: Graph | undefined = graph;
	while (node) {
		names.unshift(node.Krate.name);
		node = node.parents?.[0];
	}
	return names.join(" › ");
}

/**
 * Runs `cargo deny` and `cargo shear` on the checkout at `root`. Each side uses
 * its own `deny.toml`, so a PR that fixes a finding through config shows it as
 * fixed. Neither tool compiles anything.
 */
export class UnknownDenyCode extends Schema.TaggedError<UnknownDenyCode>()("UnknownDenyCode", {
	code: Schema.String,
}) {
	override get message() {
		return `cargo-deny reported the unknown diagnostic code \`${this.code}\`; update DENY_CODES for the pinned cargo-deny version`;
	}
}

export const collectHygiene = Effect.fn("Hygiene.collect")(function* (root: string) {
	// cargo-deny exits non-zero when it finds issues and writes its JSON to stderr,
	// so success is judged by whether it reached its summary line.
	const deny = yield* capture(
		"cargo",
		["deny", "--format", "json", "--log-level", "error", "check", "advisories", "licenses", "sources", "bans"],
		{ cwd: root },
	);
	const lines = deny.stderr.split("\n").filter((line) => line.startsWith("{"));
	const parsed = yield* Effect.forEach(lines, (line) =>
		Schema.decodeUnknownEffect(Schema.fromJsonString(DenyLine))(line).pipe(Effect.mapError(() => commandFailed(deny))),
	);
	if (!parsed.some((line) => line.type === "summary")) return yield* commandFailed(deny);

	const findings: Array<HygieneFinding> = [];
	for (const { type, fields: f } of parsed) {
		if (type !== "diagnostic" || f?.severity !== "error" || !f.code) continue;
		const check = DENY_CODES[f.code];
		if (check === undefined) return yield* new UnknownDenyCode({ code: f.code });
		const graph = f.graphs?.[0];
		const krate = graph ? `${graph.Krate.name}@${graph.Krate.version}` : "";
		const id = f.advisory?.id;
		findings.push({
			check,
			// The crate version is left out so a routine version bump of an already-flagged crate isn't "new".
			key: [f.code, id, graph?.Krate.name].filter(Boolean).join(":"),
			title: id ? `${id} ${krate}: ${f.message ?? ""}` : `${krate}: ${f.message ?? f.code}`,
			via: graph ? via(graph) : "",
		});
	}

	// cargo-shear exits 1 when it finds issues. Any other exit, or output that isn't
	// its JSON report, is a failure of the tool itself.
	const shearRun = yield* capture("cargo", ["shear", "--format", "json"], { cwd: root });
	const shear = yield* decodeReport(shearRun, ShearOutput);
	for (const f of shear.findings) {
		if (f.severity !== "error" && f.severity !== "warning") continue;
		findings.push({
			check: "unused",
			key: `${f.code}:${f.file ?? ""}:${f.message}`,
			title: f.message.split("\n")[0] ?? f.message,
			via: f.file ?? "",
		});
	}
	return findings;
});

const decodeReport = <S extends Schema.Constraint>(captured: Captured, schema: S) =>
	captured.exitCode === 0 || captured.exitCode === 1
		? Schema.decodeUnknownEffect(Schema.fromJsonString(schema))(captured.stdout).pipe(
				Effect.mapError(() => commandFailed(captured)),
			)
		: Effect.fail(commandFailed(captured));
