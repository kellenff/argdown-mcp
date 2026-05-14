/**
 * Canonical Argdown application factory.
 *
 * Builds either a sync `ArgdownApplication` (for inline string input) or an
 * `AsyncArgdownApplication` (for file-path input with `@import` resolution)
 * with an identical, minimal plugin set so the response shape is consistent
 * across both code paths.
 *
 * Deliberately avoids `@argdown/node`'s pre-configured `argdown` singleton,
 * which eagerly registers ~20 plugins (Html/Svg/Pdf/Color/Group/Tag/
 * Selection/...) and reads `argdown.config.json` from disk.
 *
 * Stage IDs (`"load-file"`, `"parse-input"`, `"build-model"`, `"export-json"`)
 * are verified against the upstream singletons:
 *   - `@argdown/core/dist/argdown.js` (parse-input, build-model, export-json)
 *   - `@argdown/node/dist/argdown.js` (load-file)
 *
 * The `(withFileIO: true, async: false)` combination is a compile-time error
 * enforced by the `BuildAppOptions` discriminated union.
 */

import {
  ArgdownApplication,
  JSONExportPlugin,
  ModelPlugin,
  ParserPlugin,
} from "@argdown/core";
import {
  AsyncArgdownApplication,
  IncludePlugin,
  LoadFilePlugin,
} from "@argdown/node";

// The (withFileIO: true, async: false) combination is a compile-time error enforced here.
export type BuildAppOptions =
  | {
      /** When true, returns an `AsyncArgdownApplication`. */
      async: true;
      /** When true, registers `JSONExportPlugin` and defines the `"export-json"` process. */
      withJson: boolean;
      /**
       * When true, registers `LoadFilePlugin` + `IncludePlugin` at the `"load-file"`
       * stage and prepends `"load-file"` to the named processes.
       */
      withFileIO: boolean;
    }
  | {
      /** When false, returns a sync `ArgdownApplication`. */
      async: false;
      /** When true, registers `JSONExportPlugin` and defines the `"export-json"` process. */
      withJson: boolean;
      /**
       * Must be `false` when `async` is `false` — `LoadFilePlugin` and
       * `IncludePlugin` are async-only (`IAsyncArgdownPlugin`) and cannot be
       * executed by a sync application.
       */
      withFileIO: false;
    };

export function buildApp(opts: {
  async: true;
  withJson: boolean;
  withFileIO: boolean;
}): AsyncArgdownApplication;
export function buildApp(opts: {
  async: false;
  withJson: boolean;
  withFileIO: false;
}): ArgdownApplication;
export function buildApp(
  opts: BuildAppOptions,
): ArgdownApplication | AsyncArgdownApplication {
  const app: ArgdownApplication | AsyncArgdownApplication = opts.async
    ? new AsyncArgdownApplication()
    : new ArgdownApplication();

  // Always-on core pipeline.
  app.addPlugin(new ParserPlugin(), "parse-input");
  app.addPlugin(new ModelPlugin(), "build-model");

  if (opts.withFileIO) {
    // Both plugins belong to the same "load-file" stage:
    //   - LoadFilePlugin reads the input file from disk.
    //   - IncludePlugin resolves `@import` directives transitively.
    app.addPlugin(new LoadFilePlugin(), "load-file");
    app.addPlugin(new IncludePlugin(), "load-file");
  }

  if (opts.withJson) {
    app.addPlugin(new JSONExportPlugin(), "export-json");
  }

  // Named processes. Compose stage lists conditionally so we never reference
  // a stage that has no registered plugins.
  const parseStages: string[] = opts.withFileIO
    ? ["load-file", "parse-input", "build-model"]
    : ["parse-input", "build-model"];

  app.defaultProcesses["parse"] = parseStages;

  if (opts.withJson) {
    app.defaultProcesses["export-json"] = [...parseStages, "export-json"];
  }

  return app;
}
