#!/usr/bin/env node
// Note: tsup's banner config also adds the shebang to dist/; this one is harmless in dev.

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { InputSchema, dispatch } from "./tools/shared.js";

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
      "Argdown argumentation toolchain. Two tools:\n" +
      "  - parse: lex/parse an Argdown document; returns diagnostics + summary counts.\n" +
      "  - export_json: parse + serialise the full AST as JSON.\n" +
      "Both tools accept either inline `source` (kind: 'inline') or a filesystem `path` (kind: 'file').\n" +
      "File-mode resolves @import directives relative to the importer.",
  },
);

server.registerTool(
  "parse",
  {
    description:
      "Parse an Argdown document and return diagnostics (lexer errors, parser errors, exceptions) plus a summary of statement/argument/section counts.",
    inputSchema: InputSchema,
    annotations: {
      readOnlyHint: true,
      openWorldHint: false,
    },
  },
  async (input) => dispatch(input as never, "parse"),
);

server.registerTool(
  "export_json",
  {
    description:
      "Parse an Argdown document and emit the full AST as a JSON string, plus the same diagnostics as `parse`.",
    inputSchema: InputSchema,
    annotations: {
      readOnlyHint: true,
      openWorldHint: false,
    },
  },
  async (input) => dispatch(input as never, "export_json"),
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
