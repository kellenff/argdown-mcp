---
name: trace-argument
description: This skill should be used when the user asks to "trace why X follows", "what supports Y", "walk the argument chain", or wants to follow the support/attack network from a named claim or argument in an Argdown document. Calls the export_json MCP tool and BFS-traverses the relation graph. SKIP: when the user asks to find unsupported premises broadly — that's find-unsupported-premises. SKIP: when the user asks to generate a counter-argument — that's rebut-argument.
---

# trace-argument

Walk the support/attack chain from a named claim. Detect cycles.

## Required MCP tools

- `mcp__argdown-mcp__export_json`

## Workflow

1. Identify the target — the user named a statement title (e.g., `[The Conclusion]`) or an argument title (e.g., `<Main Argument>`).
2. Call `export_json` on the document; extract the fenced JSON.
3. Build an adjacency view from `relations[]`:
   - Forward (supports): `r.type === "support" && r.to.title === target` ⇒ `r.from` supports target.
   - Backward (supports of supports): same, recurse on each.
   - Attacks: `r.type === "attack"` — collect separately to annotate the trace.
4. BFS from the target. Maintain a `visited: Set<title>` to detect cycles. On revisit, emit `[CYCLE: X ↔ Y]` and stop that branch.
5. Output the trace as an indented list:
   ```
   [Target]
     ← support: [Premise A]
       ← support: [Sub-claim]
       ↯ attack: [Counter-A]
     ← support: [Premise B]
   ```
6. Note cycles, dead-ends (premises with no further supports), and any `attack` edges that defeat conclusions in the chain.

## See also

- [references/argdown-json-schema.md](../references/argdown-json-schema.md) — `relations[]` shape
- [references/mcp-tool-usage.md](../references/mcp-tool-usage.md) — when to use export_json

## What this skill does NOT do

- Does not propose new premises.
- Does not generate counter-arguments.
- Does not audit ALL premises (that's find-unsupported-premises).
