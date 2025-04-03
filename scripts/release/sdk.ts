import type { ReleaseOpts } from "./main.ts";
import { assertStringIncludes } from "@std/assert";
import $ from "dax";

async function npmVersionExists(
	packageName: string,
	version: string,
): Promise<boolean> {
	$.logStep("Checking if NPM version exists", `${packageName}@${version}`);
	const npmCheck = await $`npm view ${packageName}@${version} version`.quiet()
		.noThrow();
	if (npmCheck.code === 0) {
		return true;
	} else {
		assertStringIncludes(
			npmCheck.stderr,
			`No match found for version ${version}`,
			"unexpected output",
		);
		return false;
	}
}

async function jsrVersionExists(
	packageName: string,
	version: string,
): Promise<boolean> {
	$.logStep("Checking if JSR version exists", `${packageName}@${version}`);
	const denoCheck = await $`deno info jsr:${packageName}@${version}`.quiet()
		.noThrow();
	if (denoCheck.code === 0) {
		return true;
	} else {
		assertStringIncludes(
			denoCheck.stderr,
			`Could not find version of '${packageName}' that matches specified version constraint '${version}'`,
			"unexpected output",
		);
		return false;
	}
}

export async function publishSdk(opts: ReleaseOpts) {
	const packages = [
		{
			path: `${opts.root}/sdks/api/runtime/typescript`,
			name: "@rivet-gg/api",
			npm: true,
		},
		{
			path: `${opts.root}/sdks/api/full/typescript`,
			name: "@rivet-gg/api-full",
			npm: true,
		},
		// {
		// 	path: `${opts.root}/sdks/actor/core`,
		// 	name: "@rivet-gg/actor",
		// 	//jsr: true,
		// 	npm: true,
		// 	turbo: true
		// },
		{
	
			path: `${opts.root}/frontend/packages/cli`,
			name: "@rivet-gg/cli",
			//jsr: true,
			npm: true,
			turbo: true
		}
	];

	for (const pkg of packages) {
		if(pkg.turbo) {
			await $`pnpm build --filter ${pkg.name}`;
		}

		// Check if version already exists
		let versionExists = false;
		if (pkg.npm) {
			versionExists = await npmVersionExists(pkg.name, opts.version);
		}

		if (versionExists) {
			$.logLight(
				`Version ${opts.version} of ${pkg.name} already exists. Skipping...`,
			);
			continue;
		}

		// Publish
		if (pkg.npm) {
			$.logStep("Publishing to NPM", `${pkg.name}@${opts.version}`);
			await $`pnpm install`.cwd(pkg.path);
			await $`pnpm publish --access public`.cwd(pkg.path);
		}
	}
}
