#!/usr/bin/env bash
./scripts/fern/generate_local.sh
./scripts/openapi/gen_spec_compat.sh
./scripts/openapi/gen_rust.sh
