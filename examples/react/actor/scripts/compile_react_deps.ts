import * as esbuild from "npm:esbuild";

const NodeToDenoInternals: esbuild.Plugin = {
  name: "node-to-deno",
  setup({ onResolve }) {
    onResolve({ filter: /crypto|util|async_hooks/ }, (args) => {
      return { path: `node:${args.path}`, external: true };
    });
  },
};

await esbuild.build({
  entryPoints: ["entrypoint.js"],
  conditions: ["node", "react-server"],
  external: ["util", "crypto", "async_hooks"],
  outfile: "react-server.gen.js",
  bundle: true,
  target: "node20",
  format: "esm",
  plugins: [NodeToDenoInternals],
  banner: {
    js:
      "import { createRequire } from 'node:module'; const require = createRequire(__filename);",
  },
});
