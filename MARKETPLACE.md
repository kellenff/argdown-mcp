# @casualtheorics/argdown-plugin — Claude Marketplace

**Argumentation reasoning for Claude Code.** Parse, validate, query, rebut, and compute grounded extensions over [Argdown](https://argdown.org/) documents — all without leaving your editor.

## Install

```bash
claude plugin install @casualtheorics/argdown-plugin
```

The plugin auto-registers the `@casualtheorics/argdown-mcp` server. No separate setup needed.

---

## Skills

Claude activates these automatically based on what you ask.

| Skill                       | Triggers                                                                                 |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `validate-argdown`          | "validate this argdown" · "check argdown syntax" · "is this valid argdown"               |
| `find-unsupported-premises` | "find weak points" · "find gaps in the support" · "where is X grounded"                  |
| `trace-argument`            | "trace why X follows" · "what supports Y" · "walk the argument chain"                    |
| `rebut-argument`            | "rebut this argument" · "steelman a counter" · "how would someone disagree with"         |
| `extract-argument`          | "extract the argument from this prose" · "convert this to argdown"                       |
| `argdown-to-prose`          | "summarise this argdown" · "explain this argument map in words"                          |
| `dung-extensions`           | "compute the grounded extension" · "which arguments survive" · "find rejected arguments" |

---

## What you get

### Syntax validation

Parses Argdown and reports lexer/parser errors with `line:col` positions. Never rewrites — advises only.

### Structural query

Surface premises with no inbound support. Applies Govier's ARG check (Acceptability / Relevance / Grounds). BFS through the relation graph with cycle detection.

### Scheme-aware rebuttal

Generates counter-positions using Pollock's frame (rebutting vs undercutting). Classifies target arguments against Walton's 8-scheme catalogue and presses the matching critical question.

### Dung grounded extension

Computes the IN / OUT / UNDEC partition over the document's arguments under Dung's abstract argumentation framework using Caminada's three-valued labelling algorithm. Handles cycles, reinstatement, and self-attack correctly.

### Extraction and prose

Decomposes free-form argumentative prose into Argdown with Toulmin role labels. Renders Argdown maps as flowing English prose preserving relation semantics.

---

## MCP server

The plugin ships with `.mcp.json` wired to `@casualtheorics/argdown-mcp`, which exposes three tools:

| Tool              | Purpose                                                                           |
| ----------------- | --------------------------------------------------------------------------------- |
| `parse`           | Parse + validate; returns structural summary and diagnostics                      |
| `export_json`     | Full `IArgdownResponse.json` payload — statements, arguments, relations, sections |
| `dung_extensions` | Grounded extension: IN / OUT / UNDEC partition + counts                           |

---

## Packages

| Package       | npm                                                                                              | Version |
| ------------- | ------------------------------------------------------------------------------------------------ | ------- |
| MCP server    | [`@casualtheorics/argdown-mcp`](https://www.npmjs.com/package/@casualtheorics/argdown-mcp)       | 0.2.0   |
| Claude plugin | [`@casualtheorics/argdown-plugin`](https://www.npmjs.com/package/@casualtheorics/argdown-plugin) | 0.2.0   |

## Platform

Linux and macOS. Windows is not supported.

## License

MIT — Casual Theorics
