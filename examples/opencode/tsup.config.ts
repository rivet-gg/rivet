import { defineConfig } from "tsup";

export default defineConfig({
	entry: ["src/server.ts", "src/host.ts"],
	format: ["esm"],
	platform: "node",
	target: "node22",
	clean: true,
	sourcemap: true,
	// The preview SDK publishes extensionless internal imports. Bundle its
	// entry points so this example runs in plain Node.js.
	noExternal: [/^@opencode\/(?!util(?:\/|$))/, /^@parcel\/watcher\/wrapper$/],
	shims: true,
	removeNodeProtocol: false,
	banner: {
		js: 'import { createRequire as __rivetCreateRequire } from "node:module"; const require = __rivetCreateRequire(import.meta.url);',
	},
	esbuildPlugins: [
		{
			name: "opencode-native-assets",
			setup(build) {
				const providers: Record<string, string> = {
					"#fff": "filesystem/fff.node",
					"#pty": "pty/pty.node",
					"#shell-parser-wasm": "shell/parser-wasm.node",
					"#photon-wasm": "image/photon-wasm.node",
					"#persistent-pty-binary": "persistent-pty/binary.node",
					"#process-lock-ffi": "util/process-lock-ffi.node",
				};
				build.onResolve({ filter: /^#/ }, ({ path }) =>
					providers[path]
						? {
								path: `@opencode/core/${providers[path]}`,
								external: true,
							}
						: undefined,
				);
			},
		},
	],
});
