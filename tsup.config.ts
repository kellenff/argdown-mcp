import { defineConfig } from "tsup";
export default defineConfig({
  entry: ["src/server.ts"],
  format: ["esm"],
  target: "node24",
  platform: "node",
  clean: true,
  dts: false,
  // Bundle everything except Node builtins. The banner below provides a
  // createRequire shim so CJS deps (cosmiconfig, import-fresh, etc.) that
  // use dynamic require() resolve correctly in our ESM output.
  noExternal: [/.*/],
  external: ["node:*"],
  outDir: "dist",
  shims: true, // __dirname/__filename/import.meta.url shims
  banner: {
    js: [
      "#!/usr/bin/env node",
      // CJS deps (cosmiconfig, import-fresh, etc.) call require() dynamically.
      // Provide it via createRequire so the ESM bundle can satisfy those calls.
      'import { createRequire as __mcpCreateRequire } from "node:module";',
      "globalThis.require = globalThis.require ?? __mcpCreateRequire(import.meta.url);",
    ].join("\n"),
  },
  sourcemap: false,
});
