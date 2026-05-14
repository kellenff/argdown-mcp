/**
 * Parse a `.argdown` file from disk with transitive `@import` resolution.
 *
 * Thin wrapper around the async `buildApp` pipeline: `load-file` →
 * `parse-input` → `build-model` (+ optional `export-json`).
 *
 * Diagnostics — including a missing file — flow through `response.exceptions`
 * rather than throwing, because `throwExceptions: false` causes the
 * `AsyncArgdownApplication` to project `ArgdownPluginError`s (which is what
 * `LoadFilePlugin` raises for `ENOENT`) into the response.
 */

import path from "node:path";
import {
  ArgdownPluginError,
  type IArgdownRequest,
  type IArgdownResponse,
} from "@argdown/core";
import { buildApp } from "./build-app.js";

export type RunFileOptions = {
  withJson: boolean;
};

export async function runFile(
  filePath: string,
  opts: RunFileOptions,
): Promise<IArgdownResponse> {
  const app = buildApp({
    async: true,
    withJson: opts.withJson,
    withFileIO: true,
  });

  // The path must be absolute so that `IncludePlugin`'s `@import` resolution,
  // which resolves relative to the importer, has a stable anchor for the
  // initial file. Verified against
  // `node_modules/@argdown/node/dist/plugins/LoadFilePlugin.js`: the plugin
  // reads `request.inputPath` (NOT `request.input`, which it then *writes*
  // with the file contents).
  const absolutePath = path.resolve(filePath);

  const request: IArgdownRequest = {
    inputPath: absolutePath,
    process: opts.withJson ? "export-json" : "parse",
    throwExceptions: false,
    logExceptions: false,
  };

  try {
    return await app.runAsync(request);
  } catch (e) {
    // Defensive: `throwExceptions: false` causes `ArgdownPluginError`s to be
    // captured in `response.exceptions`, but non-Argdown errors are otherwise
    // swallowed silently. Project anything that *does* escape into a
    // synthetic response so callers always see a uniform shape.
    const err =
      e instanceof Error
        ? e
        : new ArgdownPluginError("runFile", "unknown-error", String(e));
    return { exceptions: [err] };
  }
}
