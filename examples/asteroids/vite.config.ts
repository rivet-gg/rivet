import { defineConfig } from "vite";

import { nodePolyfills } from "vite-plugin-node-polyfills";
import tsconfigPaths from "vite-tsconfig-paths";
import EnvironmentPlugin from 'vite-plugin-environment';

export default defineConfig({
    plugins: [
        tsconfigPaths(),
        nodePolyfills({ globals: { Buffer: true } }),
        EnvironmentPlugin(['ACTOR_CORE_ENDPOINT', '_LOG_LEVEL']),
    ],
    build: {
        outDir: "./build/client/",
        emptyOutDir: true,
    },
});
