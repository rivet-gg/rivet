import type { ReleaseOpts } from "./main.ts";
import { assertExists } from "@std/assert/exists";
import $, { CommandBuilder } from "dax";
import { assert } from "@std/assert/assert";

export async function updateArtifacts(opts: ReleaseOpts) {
    // Get credentials and set them in the environment
    const awsAccessKeyId = await $`op read "op://Engineering/rivet-releases R2 Upload/username"`.text();
    const awsSecretAccessKey = await $`op read "op://Engineering/rivet-releases R2 Upload/password"`.text();

    // Create AWS CLI command builder with credentials
    const awsCommand = new CommandBuilder()
        .env({
            AWS_ACCESS_KEY_ID: awsAccessKeyId,
            AWS_SECRET_ACCESS_KEY: awsSecretAccessKey,
            AWS_ENDPOINT_URL: "https://2a94c6a0ced8d35ea63cddc86c2681e7.r2.cloudflarestorage.com",
            AWS_DEFAULT_REGION: "auto"
        });

    // List all files under rivet/{commit}/
    const commitPrefix = `rivet/${opts.commit}/`;
    $.logStep("Listing Original Files", commitPrefix);
    const commitFiles = await awsCommand.command(`aws s3api list-objects --bucket rivet-releases --prefix ${commitPrefix}`).json();
    assertExists(commitFiles?.Contents);
    assert(commitFiles.Contents.length > 0, `No files found under rivet/${opts.commit}/`);

    // Copy files to version directory
    const versionTarget = `rivet/${opts.version}/`;
    await copyFiles(awsCommand, commitPrefix, versionTarget);

    // If this is the latest version, copy to latest directory
    if (opts.latest) {
        await copyFiles(awsCommand, commitPrefix, "rivet/latest/");
    }
}

async function copyFiles(
    awsCommand: CommandBuilder,
    sourcePrefix: string,
    targetPrefix: string,
) {
    $.logStep("Copying Files", targetPrefix);
    await $.logGroup(async () => {
        // Delete existing files in target directory using --recursive
        $.logStep("Deleting existing files in", targetPrefix);
        await awsCommand.command(`aws s3 rm s3://rivet-releases/${targetPrefix} --recursive`).spawn();

        // Copy new files using --recursive
        $.logStep("Copying files from", sourcePrefix, "to", targetPrefix);
        await awsCommand.command(`aws s3 cp s3://rivet-releases/${sourcePrefix} s3://rivet-releases/${targetPrefix} --recursive --copy-props none`).spawn();
    });
}
