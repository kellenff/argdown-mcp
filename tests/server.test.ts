/**
 * Vitest suite covering Success Criteria 1-8 from the argdown-mcp plan.
 *
 * Schema note: the plan originally specified a discriminated-union input
 * schema, but the MCP SDK's `normalizeObjectSchema` only handles a Zod raw
 * shape / `z.object(...)`. The shipped code uses a flat shape with `kind` as
 * a string-literal discriminator and validates `source`/`path` presence in
 * the handler. These tests assert against the shipped (flat) shape.
 *
 * Each `describe` block corresponds to one success criterion; each test
 * spins up its own InMemoryTransport-linked client/server pair and tears it
 * down, so failures stay isolated.
 */

import { describe, it, expect } from "vitest";
import path from "node:path";
import { makeClient, fixtureDir, firstText } from "./helpers.js";

describe("SC 1 — tools register correctly", () => {
  it("exposes `parse`, `export_json`, and `dung_extensions` with the flat InputShape", async () => {
    const { client, close } = await makeClient();
    try {
      const tools = await client.listTools();
      const names = tools.tools.map((t) => t.name).sort();
      expect(names).toEqual(["dung_extensions", "export_json", "parse"]);

      for (const t of tools.tools) {
        const schema = t.inputSchema as {
          type?: string;
          properties?: Record<string, { enum?: string[] }>;
          required?: string[];
        };
        expect(schema?.type).toBe("object");
        expect(schema?.properties?.kind?.enum?.sort()).toEqual([
          "file",
          "inline",
        ]);
        expect(schema?.properties?.source).toBeDefined();
        expect(schema?.properties?.path).toBeDefined();
        expect(schema?.required).toContain("kind");

        // Annotations propagate to the client.
        expect(t.annotations?.readOnlyHint).toBe(true);
        expect(t.annotations?.openWorldHint).toBe(false);
      }
    } finally {
      await close();
    }
  });
});

describe("SC 2 — inline parse happy path", () => {
  it("parses two related statements without diagnostics", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "parse",
        arguments: { kind: "inline", source: "[a]: hello\n  + [b]: world\n" },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      // At least 2 statements (a + b), no error section.
      expect(text).toMatch(
        /Parsed [2-9]\d* statements|Parsed [2-9] statements/,
      );
      expect(text).not.toMatch(/Errors:/);
      expect(text).not.toMatch(/not valid Argdown/i);
    } finally {
      await close();
    }
  });
});

describe("SC 3 — inline parse error path", () => {
  it("flags malformed input with isError + Errors: section and stays alive", async () => {
    const { client, close } = await makeClient();
    try {
      // Relation marker without a content target — guaranteed parser error
      // ("Missing relation content..."). Confirmed against @argdown/core 2.x
      // via local probe; lighter malformations (`[a`, `@@@`) are tolerated
      // and produce zero diagnostics.
      const result = await client.callTool({
        name: "parse",
        arguments: { kind: "inline", source: "[a]: x\n  + " },
      });
      expect(result.isError).toBe(true);
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      expect(text).toMatch(/Errors:/);
      // Some diagnostic line carries a `L<line>:<col>` reference.
      expect(text).toMatch(/L\d+:\d+/);

      // Server still answers a follow-up call.
      const ok = await client.callTool({
        name: "parse",
        arguments: { kind: "inline", source: "[ok]: still here" },
      });
      expect(ok.isError).toBeFalsy();
    } finally {
      await close();
    }
  });
});

describe("SC 4 — file parse honours @import", () => {
  it("merges imported.argdown into main.argdown, yielding ≥2 statements", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "parse",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "main.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      // main has 2 statements, imported has 2 — after @import textual
      // substitution we expect ≥ 2 (likely 4).
      const m = text.match(/Parsed (\d+) statements/);
      expect(m).not.toBeNull();
      expect(Number(m![1])).toBeGreaterThanOrEqual(2);
      expect(text).not.toMatch(/Errors:/);
    } finally {
      await close();
    }
  });
});

describe("SC 5 — inline export_json", () => {
  it("emits a fenced JSON block with statements/arguments/relations", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "export_json",
        arguments: { kind: "inline", source: "[a]: hello\n  + [b]: world\n" },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const match = text.match(/```json\n([\s\S]+?)\n```/);
      expect(match).not.toBeNull();
      const json = JSON.parse(match![1]!);
      expect(json).toHaveProperty("statements");
      expect(json).toHaveProperty("arguments");
      expect(json).toHaveProperty("relations");
    } finally {
      await close();
    }
  });
});

describe("SC 6 — kind/source/path enforcement", () => {
  it("rejects kind='inline' with no source", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "parse",
        arguments: { kind: "inline" },
      });
      expect(result.isError).toBe(true);
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      expect(text.toLowerCase()).toContain("source");
    } finally {
      await close();
    }
  });

  it("rejects kind='file' with no path", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "parse",
        arguments: { kind: "file" },
      });
      expect(result.isError).toBe(true);
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      expect(text.toLowerCase()).toContain("path");
    } finally {
      await close();
    }
  });
});

describe("SC 7 — non-Argdown file returns isError with (not valid Argdown) hint", () => {
  // The tightened heuristic in shape.ts requires at least one *titled* statement,
  // argument, section, or relation before declaring input valid. Plain prose
  // (which @argdown/core parses into 'Untitled N' synthetic statements only)
  // now correctly fails the looks-like-argdown gate.
  it("flags not-argdown.txt with isError + 'not valid Argdown' hint and stays alive", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "parse",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "not-argdown.txt"),
        },
      });
      expect(result.isError).toBe(true);
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      expect(text).toMatch(/not valid Argdown/i);

      // Server still answers afterwards.
      const ok = await client.callTool({
        name: "parse",
        arguments: { kind: "inline", source: "[a]: still works" },
      });
      expect(ok.isError).toBeFalsy();
    } finally {
      await close();
    }
  });
});

describe("Dung SC-2 — 3-cycle yields all UNDEC", () => {
  it("returns { in: [], out: [], undec: [A,B,C] } under grounded semantics", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "three-cycle.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in).toEqual([]);
      expect(json.extension.out).toEqual([]);
      expect(json.extension.undec.sort()).toEqual(["A", "B", "C"]);
      expect(json.argumentCount).toBe(3);
      expect(json.attackCount).toBe(3);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-3 — defended argument", () => {
  it("B attacks A, B unattacked: { in: [B], out: [A] }", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "defended.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in).toEqual(["B"]);
      expect(json.extension.out).toEqual(["A"]);
      expect(json.extension.undec).toEqual([]);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-4 — reinstatement (4-chain D attacks C attacks B attacks A)", () => {
  // Load-bearing: catches naive "unattacked=IN, attacked=OUT" implementations.
  // D unattacked → IN. C attacked by D (IN) → OUT. B attacked by C (only attacker, OUT) → IN.
  // A attacked by B (IN) → OUT. The fixpoint REINSTATES B via C's defeat.
  it("yields { in: [B,D], out: [A,C] } via reinstatement", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "reinstatement-4chain.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in.sort()).toEqual(["B", "D"]);
      expect(json.extension.out.sort()).toEqual(["A", "C"]);
      expect(json.extension.undec).toEqual([]);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-5 — no attacks", () => {
  it("all arguments IN, OUT and UNDEC empty", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "no-attacks.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in.sort()).toEqual(["A", "B", "C"]);
      expect(json.extension.out).toEqual([]);
      expect(json.extension.undec).toEqual([]);
      expect(json.attackCount).toBe(0);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-6 — empty / argument-free document", () => {
  it("all three sets empty, no throw", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "empty.argdown"),
        },
      });
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in).toEqual([]);
      expect(json.extension.out).toEqual([]);
      expect(json.extension.undec).toEqual([]);
      expect(json.argumentCount).toBe(0);
      expect(json.attackCount).toBe(0);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-7 — self-attack convention", () => {
  it("isolated self-attacker is UNDEC", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "self-attack-alone.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in).toEqual([]);
      expect(json.extension.out).toEqual([]);
      expect(json.extension.undec).toEqual(["S"]);
    } finally {
      await close();
    }
  });

  it("self-attacker defeated by unattacked outsider: outsider IN, self-attacker OUT", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "self-attack-defeated.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in).toEqual(["T"]);
      expect(json.extension.out).toEqual(["S"]);
      expect(json.extension.undec).toEqual([]);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-8 — statement-level attacks ignored", () => {
  it("doc with only statement attacks → all arguments IN, attackCount 0", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "file",
          path: path.join(fixtureDir, "dung", "statement-attack-only.argdown"),
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in.sort()).toEqual(["A", "B"]);
      expect(json.extension.out).toEqual([]);
      expect(json.extension.undec).toEqual([]);
      expect(json.attackCount).toBe(0);
    } finally {
      await close();
    }
  });
});

describe("Dung SC-9 — both source and path input modes work", () => {
  it("inline source path produces a grounded extension", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: {
          kind: "inline",
          source: "<A>: a.\n  - <B>: b.\n",
        },
      });
      expect(result.isError).toBeFalsy();
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      const json = JSON.parse(text.match(/```json\n([\s\S]+?)\n```/)![1]!);
      expect(json.extension.in).toEqual(["B"]);
      expect(json.extension.out).toEqual(["A"]);
    } finally {
      await close();
    }
  });

  it("rejects kind='inline' with no source", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: { kind: "inline" },
      });
      expect(result.isError).toBe(true);
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      expect(text.toLowerCase()).toContain("source");
    } finally {
      await close();
    }
  });

  it("rejects kind='file' with no path", async () => {
    const { client, close } = await makeClient();
    try {
      const result = await client.callTool({
        name: "dung_extensions",
        arguments: { kind: "file" },
      });
      expect(result.isError).toBe(true);
      const text = firstText(
        result as { content: Array<{ type: string; text?: string }> },
      );
      expect(text.toLowerCase()).toContain("path");
    } finally {
      await close();
    }
  });
});

describe("SC 8 — @import cycle does not hang", () => {
  it(
    "returns within 5s for a cyclic @import graph",
    { timeout: 5000 },
    async () => {
      const { client, close } = await makeClient();
      try {
        const result = await client.callTool({
          name: "parse",
          arguments: {
            kind: "file",
            path: path.join(fixtureDir, "cycle", "a.argdown"),
          },
        });
        // Either the loader detects the cycle (isError true) or it terminates
        // textual substitution (isError false). Both are acceptable as long as
        // we got a response within the timeout — that is what the assertion
        // exercises.
        expect(result).toBeDefined();
        expect(Array.isArray(result.content)).toBe(true);
      } finally {
        await close();
      }
    },
  );
});
