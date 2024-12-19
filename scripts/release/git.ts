import type { ReleaseOpts } from "./main.ts";
import $ from "dax";

export async function validateGit(opts: ReleaseOpts) {
	// Validate there's no uncommitted changes
	const status = await $`git status --porcelain`.text();
	if (status) {
		throw new Error("There are uncommitted changes. Please commit or stash them.");
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
