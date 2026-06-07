---
name: argdown-analysis
description: Use when analyzing argument structure, building or inspecting dialectical maps, or asking which arguments survive/"win" in a debate — i.e. working with Argdown documents or argumentation. Routes to the argdown MCP tools (parse, export_model, dung_extensions).
---

# Argdown Analysis

The `argdown` MCP server exposes three tools over Argdown source. Prefer passing the document inline as `source`.

- **`parse`** — validate syntax and get block counts (headings, statements, arguments, relations, PCS). A parse failure returns a human-readable message plus a byte `offset`. Use it first as a cheap well-formedness check.
- **`export_model`** — the resolved Layer B model as JSON: statement and argument equivalence classes, premise-conclusion (PCS) roles, dialectical edges, and conflicts. Use it to reason about structure — what supports or attacks what, and how arguments are built.
- **`dung_extensions`** — the grounded extension under Dung's abstract argumentation framework: the unique IN / OUT / UNDEC partition of the arguments once all attacks resolve. Use it to answer "which arguments survive / are accepted?"

## When to use

- The user pastes or references an Argdown document and wants it validated, mapped, or evaluated.
- The user asks which position "wins", which arguments are defeated, or for the accepted set of a debate → `dung_extensions`.
- The user wants the structure (premises, conclusions, supports, attacks) → `export_model`.

Validate with `parse` first, then call `export_model` and/or `dung_extensions` as the question demands.
