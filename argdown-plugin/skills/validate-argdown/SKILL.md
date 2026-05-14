---
name: validate-argdown
description: This skill should be used when the user asks to "validate this argdown", "check argdown syntax", "is this valid argdown", or pastes Argdown source for review. Calls the parse MCP tool and reports diagnostics. SKIP: when the user asks to find weak premises or gaps in the argument — that's find-unsupported-premises. SKIP: when the user asks to summarise or explain an argdown document — that's argdown-to-prose.
---

# validate-argdown

Validate Argdown source. Advise; never auto-fix.

## Required MCP tools

- `mcp__argdown-mcp__parse`

## Workflow

1. Receive Argdown source (inline string or a file path).
2. Call `parse` with `{ kind: "inline" | "file", source | path }` — see [references/mcp-tool-usage.md](../references/mcp-tool-usage.md).
3. Read the text payload:
   - First line: `Parsed N statements (by source-location), M arguments, K sections.`
   - If an `Errors:` block follows, each line shows lexer/parser errors with line/col.
   - If the payload ends with `(not valid Argdown)`, the parser tolerated input but found no titled statements/arguments/sections/relations.
4. Report to the user:
   - On success: confirm parse with the summary counts.
   - On parser/lexer errors: quote each error verbatim with its line/col; suggest a fix in prose without rewriting the source.
   - On `(not valid Argdown)`: ask if the user meant to paste prose (suggest `extract-argument`) or if they expected this to parse.

## Output style

- Plain prose with bullet points for multi-error cases.
- Cite line/col when referring to errors.
- If suggesting a fix, describe the change; do not produce a rewritten Argdown block (that's outside this skill's remit — the user owns edits).

## What this skill does NOT do

- Does not call `export_json` (that's for structural reasoning skills).
- Does not modify or rewrite the source.
- Does not classify "syntactically valid but logically weak" — that's `find-unsupported-premises`.

## See also

- [references/mcp-tool-usage.md](../references/mcp-tool-usage.md)
- [references/argdown-json-schema.md](../references/argdown-json-schema.md)
