import { resolve } from "node:path";
import tsconfigPaths from "vite-tsconfig-paths";
import { configDefaults, defineConfig } from "vitest/config";
import defaultConfig from "../../../vitest.base.ts";

export default defineConfig({
	...defaultConfig,
	test: {
		...defaultConfig.test,
		exclude: [
			...configDefaults.exclude,
			...(process.env.RIVETKIT_INCLUDE_STREAM_TESTS === "1"
				? []
				: ["tests/actor-streams.integration.test.ts"]),
			...(process.env.RIVETKIT_INCLUDE_PLATFORM_TESTS === "1"
				? []
				: ["tests/platforms/**/*.test.ts"]),
		],
		fileParallelism: false,
		testTimeout: 30_000,
		hookTimeout: 30_000,
		minWorkers: 1,
		maxWorkers: 1,
		sequence: {
			...defaultConfig.test.sequence,
			concurrent: false,
		},
	},
	// Used to resolve "rivetkit" to "src/mod.ts" in the test fixtures
	plugins: [tsconfigPaths()],
	resolve: {
		alias: {
			"@": resolve(__dirname, "./src"),
			"rivetkit/errors": resolve(__dirname, "./src/actor/errors.ts"),
		},
	},
});
