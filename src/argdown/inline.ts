/**
 * Inline Argdown parser/JSON-exporter.
 *
 * Parses an Argdown source string directly (no file I/O, no `@import`
 * resolution). Uses the sync `ArgdownApplication` built by `buildApp` with
 * `withFileIO: false`, optionally with the JSON export plugin enabled.
 *
 * Exceptions raised by Argdown plugins are returned in `response.exceptions`
 * rather than thrown across this function boundary (`throwExceptions: false`).
 */

import type { IArgdownRequest, IArgdownResponse } from "@argdown/core";
import { buildApp } from "./build-app.js";

export type RunInlineOptions = {
  /** When true, run the `"export-json"` process; otherwise run `"parse"`. */
  withJson: boolean;
};

export function runInline(
  source: string,
  opts: RunInlineOptions,
): IArgdownResponse {
  const app = buildApp({
    async: false,
    withJson: opts.withJson,
    withFileIO: false,
  });

  const request: IArgdownRequest = {
    input: source,
    process: opts.withJson ? "export-json" : "parse",
    throwExceptions: false,
  };

  return app.run(request);
}

export type { IArgdownResponse } from "@argdown/core";
