---
name: extract-argument
description: This skill should be used when the user asks to "extract the argument from this prose", "convert this to argdown", "decompose this into premises and conclusions", or pastes free-form argumentative prose and asks for it as structured Argdown. Applies Toulmin's claim/data/warrant/backing frame and validates the output via the parse MCP tool. SKIP: when the user asks to summarise an existing Argdown document — that's argdown-to-prose. SKIP: when the user asks to validate Argdown they already wrote — that's validate-argdown.
---

# extract-argument

Decompose prose into Argdown using Toulmin's frame.

## Required MCP tools

- `mcp__argdown-mcp__parse` (for post-draft validation only)

## Workflow

1. Read the user's prose.
2. Apply Toulmin's frame (see [references/argumentation-theory.md](../references/argumentation-theory.md)):
   - **Claim** — the conclusion being argued for. Mark with `// role: claim` line-comment beside the statement title.
   - **Data** — evidence cited as ground for the claim. Mark `// role: data`.
   - **Warrant** — the inferential bridge (often unstated; surface it explicitly). Mark `// role: warrant`.
   - **Backing** — support for the warrant itself (often a higher-order argument). Mark `// role: backing`.
3. Render as Argdown:
   - `[Statement Title]: body` for statements (data, warrant, backing, claim).
   - `<Argument Title>: gloss` for the central argument that bundles the steps.
   - `+ [Other]` for supports; `- [Other]` for attacks.
   - Include `// role: claim|data|warrant|backing` line-comments next to each statement.
4. Validate: call `parse` with the drafted Argdown as inline `source`. If `isError: true`, fix the syntax errors quoted in the diagnostic block and re-validate. If still failing after one fix attempt, surface the error to the user and ask for guidance.
5. Output the final validated Argdown plus a brief prose summary noting which statements play which Toulmin role.

## See also

- [references/argumentation-theory.md](../references/argumentation-theory.md) — Toulmin
- [references/argdown-json-schema.md](../references/argdown-json-schema.md)
- [references/mcp-tool-usage.md](../references/mcp-tool-usage.md)

## What this skill does NOT do

- Does not modify existing Argdown (validate-argdown / argdown-to-prose territory).
- Does not generate counter-arguments (rebut-argument).
