---
name: argdown-to-prose
description: This skill should be used when the user asks to "summarise this argdown", "explain this argument map in words", "render this argdown as prose", or wants a flowing-English summary of an existing Argdown document. Calls the export_json MCP tool and walks sections in document order. SKIP: when the user asks to convert prose into Argdown — that's extract-argument. SKIP: when the user asks to trace a specific claim's support chain — that's trace-argument.
---

# argdown-to-prose

Render an Argdown document as flowing English. Preserve relation semantics.

## Required MCP tools

- `mcp__argdown-mcp__export_json`

## Workflow

1. Receive Argdown source (inline or file path).
2. Call `export_json`; extract the fenced JSON.
3. Walk `sections[]` in array order (document order is preserved). For each section, recurse into `children`.
4. For each section: render the heading as a paragraph lead; then summarise the arguments and statements that fall within the section's span (use `startLine`/`endLine` on member statements/arguments to attribute, or fall back to enumerating all top-level arguments + statements once if section spans aren't clear).
5. Convert relation semantics to English connectives:
   - `support` → "because" / "since" / "given that"
   - `attack` → "however" / "but" / "against this"
   - Other relation types — describe explicitly ("entails", "contradicts").
6. Produce one or more flowing paragraphs (not bullet lists). Quote statement titles inline with their bodies. Use the warrant/backing roles (if `// role:` comments are present, see extract-argument) to phrase inferences naturally.

## See also

- [references/argdown-json-schema.md](../references/argdown-json-schema.md) — `sections[]`, `relations[]`
- [references/mcp-tool-usage.md](../references/mcp-tool-usage.md)

## What this skill does NOT do

- Does not edit the source.
- Does not extract arguments from prose (that's the reverse — extract-argument).
- Does not trace specific claims (trace-argument).
