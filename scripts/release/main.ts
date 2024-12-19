#!/usr/bin/env -S deno run -A

import { copy } from "jsr:@std/fs";
import { resolve } from "jsr:@std/path";
import { assert, assertEquals } from "jsr:@std/assert";
import { S3Bucket } from "https://deno.land/x/s3@0.5.0/mod.ts";
// import { updateArtifacts } from "./artifacts.ts";
import { publishSdk } from "./sdk.ts";
// import { configureReleasePlease } from "./please.ts";
// import { tagDocker } from "./docker.ts";
import { validateGit } from "./git.ts";
import { parseArgs } from "jsr:@std/cli";
import $ from "dax";

const ROOT_DIR = resolve(import.meta.dirname!, "..", "..");

function getRequiredEnvVar(name: string): string {
	const value = Deno.env.get(name);
	if (!value) {
		throw new Error(`Required environment variable ${name} is not set`);
	}
	return value;
}

export interface ReleaseOpts {
	root: string;
	version: string;
	latest: boolean;
	commit: string;
}

async function main() {
	// Parse args
	const args = parseArgs(Deno.args, {
		boolean: ["latest"],
		negatable: ["latest"],
		default: {
			latest: true,
		},
	});
	assertEquals(args._.length, 1, "expected version");
	const [version] = args._ as string[];

	assert(
		/^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/
			.test(version),
		"version must be a valid semantic version starting with 'v'",
	);

	// Setup opts
	const commit = await $`git rev-parse HEAD`.text();

	const opts: ReleaseOpts = {
		root: ROOT_DIR,
		version,
		latest: args.latest,
		commit,
	};

	await validateGit(opts);
	await publishSdk(opts);
	//await tagDocker(opts);
	//await updateArtifacts(opts);
	//await configureReleasePlease(opts);
}

if (import.meta.main) {
	main();
}

// More steps:
// - Pull credentials from 1Password or something
// - Update version
//	- Default version in initiated project
//	- Version for docs
//	- Install instructions in readme
// - Docker images & docker monolith (tag with version & latest)
// - Copy release binaries to latest tag
// - Create rename commit for release please (release please will tag repo)
// - Merge release please
