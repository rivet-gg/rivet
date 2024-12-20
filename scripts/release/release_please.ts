import type { ReleaseOpts } from "./main.ts";
import $ from "dax";

export async function configureReleasePlease(opts: ReleaseOpts) {
	// Check if the Release-As commit already exists
	const commitMessage = `chore: release ${opts.version}`;
	const { code } = await $`git log --oneline --grep="${commitMessage}"`.noThrow();
	if (code === 0) {
		$.logStep("Updating Release Please", "");
		await $`git commit --allow-empty -m "${commitMessage}" -m "Release-As: ${opts.version}"`;
	} else {
		$.logLight("Release please version already configured");
	}

	// // Push changes
	// await $`git push`;
}
