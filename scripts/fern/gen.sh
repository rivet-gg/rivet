#!/usr/bin/env bash
set -euf -o pipefail

# Generate Fern libraries
set +u
if [ -n "$FERN_REPO_PATH" ]; then
	echo "Using Fern from $FERN_REPO_PATH"
	FERN_NO_VERSION_REDIRECTION=true node "$FERN_REPO_PATH/packages/cli/cli/dist/dev/cli.cjs" generate --local --group local --log-level debug
else
	fern generate --local --group local --log-level debug
fi
set -u

# Export missing types
cat <<EOF >> sdks/typescript/src/index.ts
export * as core from "./core";
export * as apiResponse from "./core/fetcher/APIResponse";
export * as fetcher from "./core/fetcher";
export * as serialization from "./serialization";
EOF

# Build libraries
#
# See https://github.com/fern-api/fern-typescript/blob/3b1c33781bbd726cee26a21c1ff3464eeae70cad/README.md?plain=1#L379
(cd sdks/typescript && yarn install && yarn build) &

# Generate OpenAPI clients
(./scripts/openapi/gen_spec_compat.sh && ./scripts/openapi/gen_rust.sh) &

wait

