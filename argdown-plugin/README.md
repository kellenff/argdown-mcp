# @casualtheorics/argdown-plugin

Claude Code plugin for reasoning about Argdown. Auto-registers the [argdown-mcp](https://github.com/kellenff/argdown-mcp) server and ships seven surgically-triggered skills for parse/validate, structural query, rebuttal, extraction, summary, and Dung grounded-extension work.

## Install

```bash
claude plugin install @casualtheorics/argdown-plugin
```

## What you get

Once installed, Claude activates one of these skills based on what you ask:

### `validate-argdown`

> "validate this argdown" · "check argdown syntax" · "is this valid argdown"

Parses your Argdown and reports syntax errors with line/col. Advises; never rewrites.

### `find-unsupported-premises`

> "find weak points in this argument" · "find gaps in the support" · "where is X grounded"

Walks the parsed structure to surface premises with no inbound support. Applies Govier's ARG check (Acceptability / Relevance / Grounds).

### `trace-argument`

> "trace why X follows" · "what supports Y" · "walk the argument chain"

BFS through the relation graph from a named claim or argument. Detects cycles. Output is an indented chain with `←` for supports and `↯` for attacks.

### `rebut-argument`

> "rebut this argument" · "steelman a counter-argument to" · "how would someone disagree with"

Generates a counter-position using Pollock's frame — rebutting (attack a premise) vs undercutting (attack the inferential warrant). Prefers undercutting when the conclusion has heavy independent support. In v0.2, classifies the target argument against Walton's argumentation-scheme catalogue (expert opinion, popular opinion, cause-to-effect, analogy, sign, consequences, example, verbal classification) and presses the scheme's matching critical question.

### `extract-argument`

> "extract the argument from this prose" · "convert this to argdown" · "decompose this into premises and conclusions"

Decomposes free-form argumentative prose into Argdown, labelling each statement with its Toulmin role (claim / data / warrant / backing). Validates the output via the parse tool before showing it.

### `argdown-to-prose`

> "summarise this argdown" · "explain this argument map in words" · "render this argdown as prose"

Walks `sections[]` in document order, renders arguments + statements as flowing English, preserves relation semantics ("because" / "however").

### `dung-extensions` (new in v0.2)

> "compute the grounded extension" · "which arguments survive" · "are these arguments acceptable" · "find rejected arguments" · "find undecided arguments"

Calls the `dung_extensions` MCP tool and reports the IN/OUT/UNDEC partition over the document's arguments under Dung's abstract argumentation framework (grounded semantics, via Caminada three-valued labelling). Only argument-to-argument attacks are considered; statement-level attacks and undercuts are intentionally ignored.

## How the MCP server is wired

`.mcp.json` declares an `argdown-mcp` server. Once the plugin loads, Claude Code starts the server automatically via `npx -y @casualtheorics/argdown-mcp`. Tools `parse`, `export_json`, and `dung_extensions` become available; the skills call them. You don't need to install or run anything else.

## Regenerating the schema reference

`skills/references/argdown-json-schema.md` is derived from a probe of the running argdown-mcp server. If `@argdown/core` updates and the JSON shape shifts, regenerate via:

```bash
cd argdown-plugin
yarn schema-probe
# diff skills/references/argdown-json-schema.md against the new .schema-probe.snapshot.json
```

## Security

Path-mode (`kind: "file"`) reads any file the MCP server's process can read. There is no sandboxing — treat its filesystem access as equivalent to any other tool you let Claude invoke. Don't run as root.

## Platform

Linux and macOS. Windows is not supported (inherited from argdown-mcp's `os: ["!win32"]`).

## License

MIT — Casual Theorics
