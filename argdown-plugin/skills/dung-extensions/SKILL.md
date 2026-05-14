---
name: dung-extensions
description: This skill should be used when the user asks to "compute the grounded extension", "which arguments survive", "are these arguments acceptable", "find rejected arguments", "find undecided arguments", or wants to resolve a graph of attacking arguments under Dung's abstract argumentation framework. Calls the dung_extensions MCP tool and reports the IN/OUT/UNDEC partition. SKIP: when the user asks to find weak premises — that's find-unsupported-premises. SKIP: when the user asks to generate a counter-argument — that's rebut-argument. SKIP: when the user asks to walk the support chain — that's trace-argument.
---

# dung-extensions

Compute which arguments survive once every attack is resolved. Apply Dung's grounded semantics.

## Required MCP tools

- `mcp__argdown-mcp__dung_extensions`

## Workflow

1. Identify the document the user is asking about (inline source or a file path).
2. Call `dung_extensions` with `kind: "inline"` (and `source`) or `kind: "file"` (and `path`). The tool reads only argument-to-argument attack relations — statement-level `>` is intentionally ignored.
3. Inspect the fenced JSON: `{ extension: { in: [...], out: [...], undec: [...] }, argumentCount, attackCount }`.
4. Report the partition in plain English:
   - **IN** — accepted: every attacker has been defeated; safe to assert.
   - **OUT** — rejected: at least one accepted argument attacks it; should not be asserted.
   - **UNDEC** — undecided: caught in a cycle or unresolved chain; the grounded semantics declines to label either way.
5. Highlight notable structures:
   - All-UNDEC over ≥3 arguments usually signals an odd-length attack cycle.
   - A single IN with many OUTs may suggest the document has a "champion" argument the rest cannot defeat.
   - Empty IN with non-empty OUT/UNDEC is rare and worth flagging.

## See also

- [references/argumentation-theory.md](../references/argumentation-theory.md) — Dung's framework and grounded semantics
- [references/mcp-tool-usage.md](../references/mcp-tool-usage.md)

## What this skill does NOT do

- Does not compute preferred / stable / complete semantics.
- Does not consider statement-level attacks (those belong to a future structured-argumentation layer).
- Does not propose new attacks or rebuttals — that's rebut-argument.
- Does not visualise the extension as a graph.
