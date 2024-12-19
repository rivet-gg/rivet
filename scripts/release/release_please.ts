import type { ReleaseOpts } from "./main.ts";
import $ from "dax";

export async function configureReleasePlease(opts: ReleaseOpts) {
	// Check if this commit already exists
	const { code } = await $`git cat-file -e ${opts.commit}`.noThrow();
	if (code === 0) {
		$.logStep("Updating Release Please", "");
		await $`git commit --allow-empty -m "chore: release ${opts.version}" -m "Release-As: ${opts.version}"`;
	} else {
		$.logLight("Release please version already configured");
	}

	// // Push changes
	// await $`git push`;
}
