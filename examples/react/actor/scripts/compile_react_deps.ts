import { rollup } from "npm:rollup";
import { env, nodeless } from "npm:@jogit/tmp-unenv";
import inject from "npm:@rollup/plugin-inject";
import virtual from "npm:@rollup/plugin-virtual";
import alias from "npm:@rollup/plugin-alias";
import commonjs from "npm:@rollup/plugin-commonjs";
import { nodeResolve } from "npm:@rollup/plugin-node-resolve";
import replace from "npm:@rollup/plugin-replace";
import * as path from "jsr:@std/path";

const unenv = env(nodeless);

export function resolveAliases(_aliases: Record<string, string>) {
  // Sort aliases from specific to general (ie. fs/promises before fs)
  const aliases = Object.fromEntries(
    Object.entries(_aliases).sort(
      ([a], [b]) =>
        b.split("/").length - a.split("/").length || b.length - a.length,
    ),
  );

  // Resolve alias values in relation to each other
  for (const key in aliases) {
    for (const alias in aliases) {
      if (aliases[key]!.startsWith(alias)) {
        aliases[key] = aliases[alias] + aliases[key]!.slice(alias.length);
      }
    }
  }

  return aliases;
}

const bundle = await rollup({
  input: "./entrypoint.js",
  external: unenv.external,
  plugins: [
    replace({
      delimiters: ["", ""],
      preventAssignment: true,
      values: {
        "process.env.NODE_ENV": '"production"',
        "typeof window": '"undefined"',
      },
    }),
    alias({
      entries: resolveAliases({
        "unenv": "@jogit/tmp-unenv",
        ...unenv.alias,
      }),
    }),
    nodeResolve({ exportConditions: ["react-server"] }),
    commonjs({
      esmExternals: (id) => !id.startsWith("unenv/"),
      requireReturnsDefault: "auto",
    }),
    inject({
      ...unenv.inject,
      setImmediate: ["unenv/runtime/node/timers/$cloudflare", "setImmediate"],
      clearImmediate: [
        "unenv/runtime/node/timers/$cloudflare",
        "clearImmediate",
      ],
    }),
    // terser({
    //   mangle: {
    //     keep_fnames: true,
    //     keep_classnames: true,
    //   },
    //   format: {
    //     comments: false,
    //   },
    // }),
  ],
});

await bundle.write({
  file: "dist/react-server.mjs",
  format: "esm",
  exports: "named",
  inlineDynamicImports: true,
  generatedCode: {
    constBindings: true,
  },
  sourcemap: false,
});
