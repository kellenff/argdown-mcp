import { defineConfig } from "tsup";
export default defineConfig({
  entry: ["src/server.ts"],
  format: ["esm"],
  target: "node24",
  platform: "node",
  clean: true,
  dts: false,
  // R1/R6 fallback: @argdown/node pulls cosmiconfig + import-fresh which use
  // dynamic require() that esbuild cannot bundle. Externalise @argdown/node
  // (consumers npm-install it; no native deps so it installs cleanly).
  // Everything else stays bundled via noExternal regex MINUS the externals.
  external: ["node:*", "@argdown/node"],
  noExternal: [
    /^@modelcontextprotocol\/sdk/,
    /^@argdown\/core/,
    /^zod/,
    /^chevrotain/,
  ],
  outDir: "dist",
  shims: true, // require() shim for transitive CJS deps (import-fresh etc.)
  banner: { js: "#!/usr/bin/env node" },
  sourcemap: false,
});
