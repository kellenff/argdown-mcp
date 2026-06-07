---
name: argdown-analysis
description: Use when analyzing argument structure, building or inspecting dialectical maps, or asking which arguments survive/"win" in a debate — i.e. working with Argdown documents or argumentation. Routes to the argdown MCP tools (parse, export_model, inspect_af, extensions, accepts).
---

# Argdown Analysis

The `argdown` MCP server exposes tools over Argdown source. Prefer passing the document inline as `source`.

- **`parse`** — validate syntax and get block counts (headings, statements, arguments, relations, PCS). A parse failure returns a human-readable message plus a byte `offset`. Use it first as a cheap well-formedness check.
- **`export_model`** — the resolved Layer B model: statement and argument equivalence classes, premise-conclusion (PCS) roles, dialectical edges, and conflicts. Optional `format`: `"json"` (default) or `"yaml"`. Use it to reason about structure — what supports or attacks what, and how arguments are built.
- **`inspect_af`** — project the Layer B model to a Dung argumentation framework: arguments, attack edges, and structural metadata (argument/attack counts, `is_acyclic`, `has_self_attacks`, strongly connected components, isolated arguments). Use it to understand the attack graph before running semantics.
- **`extensions`** — compute Dung-style extensions under the chosen semantics. Optional `semantics`: `"grounded"`, `"preferred"` (default), `"stable"`, or `"complete"`. Returns labellings (3-valued IN/OUT/UNDEC per argument) and extension sets. **Canonical tool** for "which arguments survive?" — prefer this over the deprecated alias below.
- **`accepts`** — point query: is a specific argument accepted? Requires `argument_id` (arena id). Optional `semantics` (default `"preferred"`) and `mode`: `"credulous"` (default, IN in ≥1 extension) or `"skeptical"` (IN in all extensions). Returns `accepted`, `status`, and a structured `witness` explaining why.
- **`dung_extensions`** — **DEPRECATED** — use `extensions` with `semantics: "grounded"` instead. Will be removed in v2. Returns the grounded IN/OUT/UNDEC partition only.

## When to use

- The user pastes or references an Argdown document and wants it validated, mapped, or evaluated.
- The user wants the structure (premises, conclusions, supports, attacks) → `export_model`.
- The user wants the projected attack graph or cycle/SCC structure → `inspect_af`.
- The user asks which position "wins", which arguments are defeated, or for the accepted set of a debate → `extensions` (default `semantics: "preferred"`; use `"grounded"` for the unique well-founded labelling).
- The user asks whether a **specific** argument is accepted, or credulous vs skeptical → `accepts` with `argument_id` and optional `mode`.

Validate with `parse` first, then call `inspect_af`, `export_model`, and/or `extensions` (or `accepts`) as the question demands.
