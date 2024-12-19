#!/usr/bin/env -S deno run -A

import { copy } from "jsr:@std/fs";
import { resolve } from "jsr:@std/path";
import { assert, assertEquals, assertExists } from "jsr:@std/assert";
import { S3Bucket } from "https://deno.land/x/s3@0.5.0/mod.ts";
import { publishSdk } from "./sdk.ts";
 import { configureReleasePlease } from "./release_please.ts";
import { validateGit } from "./git.ts";
import { parseArgs } from "jsr:@std/cli";
import $ from "dax";
import { tagDocker } from "./docker.ts";
import { updateArtifacts } from "./artifacts.ts";

const ROOT_DIR = resolve(import.meta.dirname!, "..", "..");

export interface ReleaseOpts {
	root: string;
	version: string;
	latest: boolean;
	/** Commit to publish release for. */
	commit: string;
}

async function main() {
	// Parse args
	const args = parseArgs(Deno.args, {
		boolean: ["latest"],
		negatable: ["latest"],
		string: ["version", "commit"],
		default: {
			latest: true,
		},
	});
	assertExists(args.version);

	assert(
		/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/.test(
			args.version,
		),
		"version must be a valid semantic version",
	);


	// Setup opts
	let commit: string;
	if (args.commit) {
		// Manually overriding commit
		commit = args.commit;
	} else {
		// Check if the current branch is 'main'
		const branch = await $`git rev-parse --abbrev-ref HEAD`.text();
		assertEquals(branch, "main", "must be on main branch to release");

		// Check if the local branch is up-to-date with the origin
		const localCommit = await $`git rev-parse HEAD`.text();
		const remoteCommit = await $`git rev-parse origin/main`.text();
		assertEquals(localCommit, remoteCommit, "your branch is not up to date with origin");

		// Save short commit
		commit = localCommit.slice(0, 7);
	}

	const opts: ReleaseOpts = {
		root: ROOT_DIR,
		version: args.version,
		latest: args.latest,
		commit,
	};

	assertEquals(opts.commit.length, 7, "must use 8 char short commit");

	await validateGit(opts);

	// TODO: Rename everything in repo

	//$.logStep("Publishing SDKs", "");
	//await $.logGroup(async () => {
	//	await publishSdk(opts);
	//});

	//$.logStep("Tagging Docker", "");
	//await $.logGroup(async () => {
	//	await tagDocker(opts);
	//});

	//$.logStep("Updating Artifacts", "");
	//await $.logGroup(async () => {
	//	await updateArtifacts(opts);
	//});

	await configureReleasePlease(opts);
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
// - Commit changes for version update in JSR/NPM
// - Merge release please
