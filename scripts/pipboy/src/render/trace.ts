/** Collapsed comment section: how a maintainer or agent pulls the full CI trace for `sha`. */
export function renderTraceHelp(sha: string): string {
	return [
		"<details><summary>🔭 Full CI trace for this commit</summary>",
		"",
		`One OpenTelemetry trace of Rust CI, frontend, and Pip-Boy for \`${sha.slice(0, 7)}\`: every job and step, with the sizes above attached. Run after CI finishes.`,
		"",
		"```bash",
		"pnpm install --frozen-lockfile --filter 'pipboy...'",
		`pnpm --filter pipboy pipboy trace --sha ${sha}`,
		"```",
		"",
		"Add `--otlp <url>` to send it to any OTLP/HTTP collector.",
		"",
		"**Agent instruction:** From a `rivet-dev/rivet` checkout, run the commands above, then read the trace file the last command prints. List the slowest jobs and steps and explain where CI time went.",
		"",
		"</details>",
	].join("\n");
}
