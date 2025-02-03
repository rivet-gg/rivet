import type { ReleaseOpts } from "./main.ts";
import $ from "dax";

export async function validateGit(_opts: ReleaseOpts) {
	// Validate there's no uncommitted changes
	const status = await $`git status --porcelain`.text();
	if (status) {
		throw new Error(
			"There are uncommitted changes. Please commit or stash them.",
		);
	}
}
