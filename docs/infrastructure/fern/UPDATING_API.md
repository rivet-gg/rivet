# Fern

## Updating the API

When an update to the API is required, a corresponding update to the Fern specification is also needed.

1. Update/Create any files you need under `gen/openapi`. For more information on how to use Fern or its syntax, see https://buildwithfern.com/docs/
2. Run `scripts/fern/gen.sh` locally. This step updates the Fern specification with the newly defined files. If this step hangs with the prompt `? Login Required. Continue?` , verify that the `FERN_TOKEN` environment variable is set on the machine.
3. From this, create a branch and commit the changes. A Github action will run to both validate and publish the new specification.

## Using the updated API

To use the newly generated API in places such as the [hub](https://github.com/rivet-gg/hub), follow the steps below

1. Find the Github action for the Fern commit
2. Go to the `fern-release-internal` job
3. Navigate to the `Releasing SDKs` task
4. Towards the end of the logs for this task, there will be a line that says `Tagging release [release tag]`.
5. Copy the release tag, and navigate to the `package.json` for your version of the hub.
6. Scroll and find `@rivet-gg/api-internal`, and replace the existing version with the release tag provided by the previous task.
7. Yarn install the new release and run the hub as normal.
