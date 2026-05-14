/**
 * Shared test scaffolding: wire an MCP server (registering the same `parse`
 * and `export_json` tools as `src/server.ts`) to a client via
 * InMemoryTransport, so tests can exercise the full request/response round-trip
 * without spawning a child process.
 *
 * The tool registrations here mirror `src/server.ts` verbatim — same names,
 * same `InputShape`, same `dispatch` calls — so any drift between this helper
 * and the production server is itself a test smell.
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { InputShape, dispatch, type Input } from "../src/tools/shared.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const pkgPath = path.resolve(__dirname, "../package.json");
const { version } = JSON.parse(readFileSync(pkgPath, "utf8")) as {
  version: string;
};

export const fixtureDir = path.resolve(__dirname, "./fixtures");

export type Harness = {
  server: McpServer;
  client: Client;
  close(): Promise<void>;
};

export async function makeClient(): Promise<Harness> {
  const server = new McpServer(
    { name: "argdown-mcp-test", version },
    {
      instructions:
        "Test instance — parse + export_json tools, identical to production server.",
    },
  );

  server.registerTool(
    "parse",
    {
      description:
        "Parse an Argdown document and return diagnostics + summary counts.",
      inputSchema: InputShape,
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async (input) => dispatch(input as Input, "parse"),
  );

  server.registerTool(
    "export_json",
    {
      description:
        "Parse an Argdown document and emit the full AST as JSON, plus diagnostics.",
      inputSchema: InputShape,
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async (input) => dispatch(input as Input, "export_json"),
  );

  const [clientTransport, serverTransport] =
    InMemoryTransport.createLinkedPair();
  const client = new Client({ name: "test-client", version: "0.1.0" });

  await Promise.all([
    server.connect(serverTransport),
    client.connect(clientTransport),
  ]);

  return {
    server,
    client,
    async close() {
      await client.close();
      await server.close();
    },
  };
}

/**
 * Convenience: text content of a tool result's first content block.
 * MCP tool results are always `{ content: [...], isError? }`; for these tools
 * the first block is always `{ type: "text", text }`.
 */
export function firstText(result: {
  content: Array<{ type: string; text?: string }>;
}): string {
  const block = result.content[0];
  if (!block || block.type !== "text" || typeof block.text !== "string") {
    throw new Error(
      `Expected first content block to be text; got ${JSON.stringify(block)}`,
    );
  }
  return block.text;
}
