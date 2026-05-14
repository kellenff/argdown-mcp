// Shebang is added by tsup's banner config (see tsup.config.ts).
// We do NOT put #!/usr/bin/env node here — the source isn't directly executed,
// and a source-level shebang would duplicate into the bundle and break ESM parse.

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { InputShape, dispatch, type Input } from "./tools/shared.js";
import { dispatchDung } from "./tools/dung.js";

// Read version from package.json (relative to source; tsup bundles this read
// such that `dist/server.js` resolves `../package.json` to the package root).
const pkgPath = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../package.json",
);
const { version } = JSON.parse(readFileSync(pkgPath, "utf8")) as {
  version: string;
};

const server = new McpServer(
  { name: "argdown-mcp", version },
  {
    instructions:
      "Argdown argumentation toolchain. Three tools:\n" +
      "  - parse: lex/parse an Argdown document; returns diagnostics + summary counts.\n" +
      "  - export_json: parse + serialise the full AST as JSON.\n" +
      "  - dung_extensions: compute the grounded extension (which arguments survive once all attacks are resolved) under Dung's abstract argumentation framework. Returns IN/OUT/UNDEC partition.\n" +
      "All tools accept either inline `source` (kind: 'inline') or a filesystem `path` (kind: 'file').\n" +
      "File-mode resolves @import directives relative to the importer.",
  },
);

server.registerTool(
  "parse",
  {
    description:
      "Parse an Argdown document and return diagnostics (lexer errors, parser errors, exceptions) plus a summary of statement/argument/section counts.",
    inputSchema: InputShape,
    annotations: {
      readOnlyHint: true,
      openWorldHint: false,
    },
  },
  async (input) => dispatch(input as Input, "parse"),
);

server.registerTool(
  "export_json",
  {
    description:
      "Parse an Argdown document and emit the full AST as a JSON string, plus the same diagnostics as `parse`.",
    inputSchema: InputShape,
    annotations: {
      readOnlyHint: true,
      openWorldHint: false,
    },
  },
  async (input) => dispatch(input as Input, "export_json"),
);

server.registerTool(
  "dung_extensions",
  {
    description:
      "Compute the grounded extension (Dung's abstract argumentation framework) over the document's arguments. " +
      "Returns the IN / OUT / UNDEC partition — which arguments survive all attacks (IN), which are defeated (OUT), and which remain undecided (UNDEC) under the unique grounded labelling. " +
      "Operates only on argument-to-argument attack relations (`<X>\\n  - <Y>`). Statement-level attacks and undercuts are ignored.",
    inputSchema: InputShape,
    annotations: {
      readOnlyHint: true,
      openWorldHint: false,
    },
  },
  async (input) => dispatchDung(input as Input),
);

// Signal handling: stdio MCP servers must exit promptly on parent disconnect / SIGPIPE.
// SIGPIPE is the common case (Claude Desktop closes the pipe).
// McpServer / StdioServerTransport handle most of this, but belt-and-braces:
process.on("SIGINT", () => process.exit(0));
process.on("SIGTERM", () => process.exit(0));
process.on("SIGPIPE", () => process.exit(0));
process.stdin.on("end", () => {
  // If transport hasn't already begun teardown, give it a beat then exit.
  setTimeout(() => process.exit(0), 100).unref();
});

const transport = new StdioServerTransport();
await server.connect(transport);
