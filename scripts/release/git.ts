import type { ReleaseOpts } from "./main.ts";
import $ from "dax";

export async function validateGit(opts: ReleaseOpts) {
	// Check if the current branch is 'main'
	const branch = await $`git rev-parse --abbrev-ref HEAD`.text();
	if (branch !== "main") {
		throw new Error("You must be on the 'main' branch to release.");
	}

	// Check if the local branch is up-to-date with the origin
	const remoteCommit = await $`git rev-parse origin/main`.text();
	if (opts.commit !== remoteCommit) {
		throw new Error("Your branch is not up-to-date with the origin/main.");
	}

	// TODO:
	// // Check if GitHub Actions workflows have run successfully
	// const token = Deno.env.get("GITHUB_TOKEN");
	// if (!token) {
	// 	throw new Error("GITHUB_TOKEN is not set in the environment.");
	// }

	// const headers = new Headers({
	// 	"Authorization": `token ${token}`,
	// 	"Accept": "application/vnd.github.v3+json",
	// });

	// const owner = "your-github-username";
	// const repo = "your-repo-name";
	// const ref = (await exec("git rev-parse HEAD")).output.trim();

	// const response = await fetch(
	// 	`https://api.github.com/repos/${owner}/${repo}/actions/runs?branch=main&status=success`,
	// 	{ headers },
	// );
	// const data = await response.json();

	// const workflows = ["build.yaml", "docker.yaml"];
	// const successfulRuns = data.workflow_runs.filter((run: any) =>
	// 	workflows.includes(run.name) && run.head_sha === ref
	// );

	// if (successfulRuns.length !== workflows.length) {
	// 	throw new Error(
	// 		"Not all required GitHub Actions workflows have run successfully.",
	// 	);
	// }
}
