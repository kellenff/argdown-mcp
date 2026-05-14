---
name: find-unsupported-premises
description: This skill should be used when the user asks to "find weak points in this argument", "find gaps in the support", "where is X grounded", or wants structural audit of which premises lack supporting evidence in an Argdown document. Calls the export_json MCP tool and walks the relation graph + PCS structure. SKIP: when the user asks to validate syntax — that's validate-argdown. SKIP: when the user asks to generate a counter-argument — that's rebut-argument.
---

# find-unsupported-premises

Identify premises that lack inbound supports. Apply Govier ARG.

## Required MCP tools

- `mcp__argdown-mcp__export_json`

## Workflow

1. Receive Argdown source (inline or path).
2. Call `export_json`; extract the fenced JSON block from the response.
3. Walk `arguments` (Record<title, IArgument>): for each, iterate `pcs[]` (premise-conclusion structure). Each row has a `role` of `"premise"` or `"conclusion"`.
4. For each premise row, look up its statement in `statements`. Check `relations[]` for any `{ type: "support", to: { title: <premise-title> } }`. Absence = an unsupported premise.
5. Apply Govier's **ARG** check (see [references/argumentation-theory.md](../references/argumentation-theory.md)) to each surfaced premise:
   - **Acceptable**: would a reasonable reader accept this without further support?
   - **Relevant**: does it connect to the conclusion?
   - **Grounded**: is it self-evident, well-known, or does it depend on a missing sub-argument?
6. Report to the user as a list of bullet items, one per premise. For each:
   - Quote the premise title.
   - State which ARG conditions look weak.
   - Suggest what kind of support would fix it (without writing the support).

## See also

- [references/argdown-json-schema.md](../references/argdown-json-schema.md) — `arguments`, `relations`, `pcs[]`
- [references/argumentation-theory.md](../references/argumentation-theory.md) — Govier ARG

## What this skill does NOT do

- Does not rewrite the source.
- Does not generate counter-arguments (rebut-argument).
- Does not validate syntax (validate-argdown).
