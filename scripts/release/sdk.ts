import type { ReleaseOpts } from "./main.ts";
import $ from "dax";

async function versionExists(
	packageName: string,
	version: string,
): Promise<boolean> {
	// Check if the version exists in the NPM registry
	const npmCheck = await $`npm view ${packageName}@${version} version`.quiet();
	if (npmCheck.stdout.trim() === version) {
		return true;
	}

	// Check if the version exists in the Deno registry
	const denoCheck = await $`deno info ${packageName}@${version}`.quiet();
	if (denoCheck.stdout.includes("version")) {
		return true;
	}

	return false;
}

export async function publishSdk(opts: ReleaseOpts) {
	const packages = [
		{ path: `${opts.root}/sdks/api/runtime`, name: "api-runtime", npm: true },
		{ path: `${opts.root}/sdks/api/full`, name: "api-full", npm: true },
		{ path: `${opts.root}/sdks/actor`, name: "actor", jsr: true },
	];

	for (const pkg of packages) {
		if (await versionExists(pkg.name, opts.version)) {
			console.log(
				`Version ${opts.version} of ${pkg.name} already exists. Skipping...`,
			);
			continue;
		}

		// Publish to NPM
		await $`npm publish --tag ${opts.version}`.cwd(pkg.path);

		// Publish to JSR (Deno)
		await $`deno publish --set-version ${opts.version}`.cwd(pkg.path);
	}
}
