# `--parallel 1` because apt has issues when pulling too many containers in parallel
docker_compose := "docker compose -f docker/dev-full/docker-compose.yml --progress=plain --parallel 1"

[group('rust')]
watch:
	bacon

[group('run')]
[no-cd]
cli *ARGS:
	cargo run -p rivet-cli -- {{ARGS}}

[group('dev')]
dev-compose *ARGS:
	{{docker_compose}} {{ARGS}}

[group('dev')]
dev-up-all:
	{{docker_compose}} up -d --build

[group('dev')]
dev-up-all-nobuild:
	{{docker_compose}} up -d

[group('dev')]
dev-up CONTAINER:
	{{docker_compose}} up -d --build {{CONTAINER}}

[group('dev')]
dev-up-nobuild CONTAINER:
	{{docker_compose}} up -d {{CONTAINER}}

[group('dev')]
dev-logs CONTAINER:
	{{docker_compose}} logs -f -n 100 {{CONTAINER}}

[group('dev')]
dev-logs-client:
	{{docker_compose}} exec rivet-client sh -c 'tail -f -n 100 /var/lib/rivet-client/logs/*'

[group('dev')]
dev-logs-client-crashed:
	docker run --rm -it -v dev-full_client-data:/var/lib/rivet-client busybox sh -c 'cat /var/lib/rivet-client/logs/*'

[group('dev')]
dev-logs-runner:
	{{docker_compose}} exec rivet-client sh -c 'tail -f -n 100 /var/lib/rivet-client/actors/*/logs/*'

[group('dev')]
dev-logs-server:
	{{docker_compose}} logs -f -n 100 rivet-server

[group('dev')]
dev-shell:
	{{docker_compose}} exec -it rivet-shell /bin/bash

[group('dev')]
dev-edge-shell:
	{{docker_compose}} exec -it rivet-edge-shell /bin/bash

[group('dev')]
dev-exec CONTAINER:
	{{docker_compose}} exec -it {{CONTAINER}} /bin/bash

[group('dev')]
dev-cmd *ARGS:
	{{docker_compose}} exec -it rivet-server rivet-server {{ARGS}}

[group('dev')]
dev-edge-cmd *ARGS:
	{{docker_compose}} exec -it rivet-edge-server rivet-edge-server {{ARGS}}

[group('dev')]
dev-down:
	{{docker_compose}} down -t 0

[group('dev')]
dev-nuke:
	{{docker_compose}} down -v -t 0

[group('fern')]
fern-check:
	./scripts/fern/check.sh

[group('fern')]
fern-gen:
	./scripts/fern/gen.sh

[group('actor')]
actor-compile-bridge:
	./scripts/sdk_actor/compile_bridge.ts

[group('actor')]
actor-check: actor-compile-bridge
	cd sdks/actor && deno check --all client/**/*.ts runtime/**/*.ts client/**/*.ts manager/**/*.ts && biome check --write

[group('system')]
system-test BUILD REGION="":
	cd examples/system-test && BUILD={{BUILD}} REGION={{REGION}} rivet shell --exec "yarn test"

[group('system')]
system-test-deploy:
	cd examples/system-test && yarn && rivet deploy

alias gcs := graphite-create-submit

[group('graphite')]
graphite-create-submit MESSAGE REVIEWER:
	gt create --all --message '{{MESSAGE}}'
	gt submit --no-edit --publish --reviewers '{{REVIEWER}}'

alias gm := graphite-modify

[group('graphite')]
graphite-modify:
	gt modify -a
	gt submit

[group('github')]
release-latest VERSION:
	./scripts/release/main.ts --setupLocal --version {{ VERSION }}
	gh workflow run .github/workflows/release.yaml -f version={{ VERSION }} -f latest=true --ref  $(git branch --show-current)
	echo 'Once workflow is complete, manually merge Release Please'

[group('github')]
release-nolatest VERSION:
	./scripts/release/main.ts --setupLocal --version {{ VERSION }} --no-latest
	gh workflow run .github/workflows/release.yaml -f version={{ VERSION }} -f latest=false --ref $(git branch --show-current)
	echo 'Once workflow is complete, manually merge Release Please'

[group('k6')]
k6-test TEST:
	k6 run --include-system-env-vars --vus 1 --iterations 1 --verbose tests/load/{{ TEST }}/index.ts

[group('k6')]
k6-run TEST:
	k6 run --include-system-env-vars tests/load/{{ TEST }}/index.ts

