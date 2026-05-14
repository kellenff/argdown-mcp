#!/usr/bin/env node
/**
 * schema-probe.mjs
 *
 * Probes the argdown-mcp server (via dist/server.js) using JSON-RPC over
 * stdio, captures the IArgdownResponse.json top-level keys + sample values,
 * and writes a deterministic snapshot to .schema-probe.snapshot.json.
 *
 * Run from argdown-plugin/:
 *   yarn schema-probe
 *
 * Idempotent — re-running overwrites cleanly.
 */

import { writeFileSync, readFileSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const pluginRoot = path.resolve(here, "..");      // argdown-plugin/

// The plugin lives inside a git worktree. The main repo (which has dist/ and
// the Yarn PnP environment) is the worktree's git-common-dir parent.
// git rev-parse --git-common-dir → e.g. /Users/.../agent-argdown/.git
const gitCommonDirResult = spawnSync("git", ["rev-parse", "--git-common-dir"], {
  cwd: pluginRoot,
  encoding: "utf8",
});
const gitCommonDir = gitCommonDirResult.stdout.trim();
const argdownRoot = path.resolve(gitCommonDir, ".."); // agent-argdown/

const serverPath = path.join(argdownRoot, "dist", "server.js");
const snapshotPath = path.join(pluginRoot, ".schema-probe.snapshot.json");
const parentPkg = JSON.parse(
  readFileSync(path.join(argdownRoot, "package.json"), "utf8"),
);
const coreVersion =
  parentPkg.devDependencies?.["@argdown/core"] ||
  parentPkg.dependencies?.["@argdown/core"] ||
  "unknown";

// Fixture exercises sections + statements + arguments + relations.
// Uses the Argdown inference-anchored notation: statements are defined first,
// then arguments reference them via support (+) and attack (-) relations.
// (The task-spec fixture omitted blank lines between blocks, causing parse errors.)
const fixture = `# Section

[Premise A]: First premise.

[Premise B]: Second premise.
  + [Premise A]

[Some target]: The target statement.

<Argument>: The conclusion.
  + [Premise A]
  + [Premise B]
  -> [Some target]
`;

// ── JSON-RPC helpers ─────────────────────────────────────────────────────────

function makeRequest(id, method, params) {
  return JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n";
}

/**
 * Send MCP JSON-RPC messages to the server and collect responses until we
 * have a response for each request ID we care about.
 */
async function runMcpSession(messages, targetIds) {
  return new Promise((resolve, reject) => {
    // Spawn with a clean environment: unset NODE_OPTIONS so the plugin's
    // Yarn PnP loader is not inherited by the server process. The server
    // (dist/server.js) is a self-contained bundle — it needs no PnP at all.
    const childEnv = { ...process.env };
    delete childEnv.NODE_OPTIONS;

    const proc = spawn(process.execPath, [serverPath], {
      stdio: ["pipe", "pipe", "pipe"],
      cwd: argdownRoot,
      env: childEnv,
    });

    const responses = new Map();
    let buffer = "";

    proc.stdout.on("data", (chunk) => {
      buffer += chunk.toString();
      const lines = buffer.split("\n");
      buffer = lines.pop(); // keep incomplete line
      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;
        try {
          const msg = JSON.parse(trimmed);
          if (msg.id !== undefined && targetIds.includes(msg.id)) {
            responses.set(msg.id, msg);
            if (responses.size === targetIds.length) {
              proc.kill("SIGTERM");
              resolve(responses);
            }
          }
        } catch {
          // ignore non-JSON lines (e.g. debug output)
        }
      }
    });

    let stderrOutput = "";
    proc.stderr.on("data", (chunk) => {
      stderrOutput += chunk.toString();
    });

    proc.on("error", reject);
    proc.on("close", (code) => {
      if (responses.size < targetIds.length) {
        reject(
          new Error(
            `Server exited (code ${code}) before all responses received. ` +
              `Got ${responses.size}/${targetIds.length}\n` +
              `stderr: ${stderrOutput}`,
          ),
        );
      }
    });

    // Write all messages to stdin (do NOT end stdin — closing stdin causes
    // the server to schedule process.exit(0) after 100ms, before we get responses).
    for (const msg of messages) {
      proc.stdin.write(msg);
    }
    // stdin remains open; we kill the process once we have all responses.
  });
}

// ── Main ─────────────────────────────────────────────────────────────────────

// MCP protocol requires an initialize handshake before calling tools.
const initRequest = makeRequest(1, "initialize", {
  protocolVersion: "2024-11-05",
  capabilities: {},
  clientInfo: { name: "schema-probe", version: "0.1.0" },
});

const initNotify = JSON.stringify({
  jsonrpc: "2.0",
  method: "notifications/initialized",
}) + "\n";

const toolRequest = makeRequest(2, "tools/call", {
  name: "export_json",
  arguments: {
    kind: "inline",
    source: fixture,
  },
});

const responses = await runMcpSession(
  [initRequest, initNotify, toolRequest],
  [1, 2],
);

const toolResponse = responses.get(2);
if (toolResponse.error) {
  throw new Error(`MCP tool call failed: ${JSON.stringify(toolResponse.error)}`);
}

// The MCP SDK wraps tool results in content[].text
const resultContent = toolResponse.result?.content;
if (!Array.isArray(resultContent) || resultContent.length === 0) {
  throw new Error(`Unexpected tool response shape: ${JSON.stringify(toolResponse.result)}`);
}

// Find the text block containing the JSON
const textBlock = resultContent.find((c) => c.type === "text");
if (!textBlock) {
  throw new Error(`No text content in tool response: ${JSON.stringify(resultContent)}`);
}

// The text may be multi-line — look for JSON after "json\n" prefix (code block)
// or parse the whole text as JSON directly.
let parsedResult;
try {
  parsedResult = JSON.parse(textBlock.text);
} catch {
  // Some servers wrap in markdown code fences; extract raw JSON
  const match = textBlock.text.match(/```(?:json)?\n([\s\S]+?)\n```/);
  if (match) {
    parsedResult = JSON.parse(match[1]);
  } else {
    throw new Error(`Cannot parse tool text as JSON:\n${textBlock.text.slice(0, 500)}`);
  }
}

// parsedResult is the shapeResponse output — check for json field
// The export_json tool returns { diagnostics, summary, json } where json is a string
let responseJson;
if (typeof parsedResult.json === "string") {
  responseJson = JSON.parse(parsedResult.json);
} else if (typeof parsedResult === "object" && parsedResult !== null) {
  // Maybe the result IS the IArgdownResponse directly
  responseJson = parsedResult;
} else {
  throw new Error(`Cannot locate IArgdownResponse JSON in: ${JSON.stringify(parsedResult).slice(0, 500)}`);
}

const topLevelKeys = Object.keys(responseJson);

function makeSample(key, value) {
  if (Array.isArray(value)) {
    return {
      type: "array",
      sample: value.slice(0, 2),
    };
  }
  if (value !== null && typeof value === "object") {
    const entries = Object.entries(value);
    return {
      type: `Record<string, ${typeof entries[0]?.[1]}>`,
      sample: entries.length > 0
        ? Object.fromEntries(entries.slice(0, 1))
        : {},
    };
  }
  return {
    type: typeof value,
    sample: value,
  };
}

const snapshot = {
  argdown_core_version: coreVersion,
  probe_input: fixture,
  top_level_keys: topLevelKeys,
  key_samples: Object.fromEntries(
    topLevelKeys.map((k) => [k, makeSample(k, responseJson[k])]),
  ),
  captured_at: new Date().toISOString(),
  captured_by: "scripts/schema-probe.mjs",
};

writeFileSync(snapshotPath, JSON.stringify(snapshot, null, 2) + "\n");
console.log(
  `wrote .schema-probe.snapshot.json (${topLevelKeys.length} top-level keys: ${topLevelKeys.join(", ")})`,
);
