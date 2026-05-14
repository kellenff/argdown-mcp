import { defineConfig } from "tsup";
export default defineConfig({
  entry: ["src/server.ts"],
  format: ["esm"],
  target: "node24",
  platform: "node",
  clean: true,
  dts: false,
  noExternal: [/.*/],
  external: ["node:*"],
  outDir: "dist",
  shims: false,
  banner: { js: "#!/usr/bin/env node" },
  sourcemap: false,
});
