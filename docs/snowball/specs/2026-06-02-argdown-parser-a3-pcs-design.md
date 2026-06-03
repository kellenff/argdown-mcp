# Argdown Parser — Increment A3 (Premise-Conclusion Structures) — Design

- **Date:** 2026-06-02
- **Status:** Approved
- **Scope:** Parse premise-conclusion structures (PCS): numbered statement
  lines, inference lines (bare `----` and ruled `-- Rule, Rule --`, capturing
  rule names), and child relations, emitted as a flat sequence of form-tagged
  items. Role assignment (premise/conclusion), inference→conclusion binding,
  relation association, and argument↔PCS linking are deferred to the semantic
  model (Layer B).

## Context: the roadmap

A (parser) shipped so far: A1 (spine — headings, plain/titled statements,
comments), A2a (arguments `<T>`/`<T>: desc` + statement references `[T]`), A2b
(relations — all 10 operators, flat with depth). This spec is **A3 (PCS)**.
Later: A4 inline, A5 metadata; then B semantic model, C JSON/Dung outputs, D
MCP server.

## Reference behavior (probed)

Confirmed against `@argdown/core` via MCP `export_json` / `parse`:

- A PCS is an ordered list of **numbered statement lines** (`(1) …`, `(2) …`),
  split by **inference lines**. The reference assigns each statement a `role`
  of `premise`, `intermediary-conclusion`, or `main-conclusion`, and attaches
  an `inference` (with `inferenceRules: [...]`) to each conclusion. Steps may
  interleave: premises → intermediary conclusion → more premises → main
  conclusion.
- **Inference-line lexing:**
  - A run of **4 or more hyphens** (`----`, `-----`) is a **bare** divider →
    `inferenceRules: []`.
  - `--` `<content>` `--` is a **ruled** divider; the content is split on commas
    into rule names (`-- Rule A, Rule B --` → `["Rule A", "Rule B"]`).
  - `---` (exactly three hyphens) is an **error** ("Please end your inference
    with two hyphens").
- Inference lines may carry a `{…}` metadata block and may themselves bear
  relations; both are **out of A3 scope** (metadata → A5).
- Roles, equivalence classes, `isUsedAsPremise`/`isUsedAsMainConclusion`, and
  the argument that owns the PCS are all **derived** by the reference. In this
  project that derivation is Layer B.

## Decisions

1. **Flat form-item representation (refined-A).** The parser emits a PCS as a
   flat `Vec<PcsItem>` in source order, where each item records *form*
   (a numbered statement, an inference line + rule names, or a relation) but
   **not role**. Premise/intermediary/main-conclusion assignment, the
   inference→conclusion binding, and relation association are Layer B's job.
   This was chosen over a role-tagged AST after an M2 brain-jam: a statement's
   role is *relational* — its meaning comes from its position in an inference
   relationship — so it belongs on the far side of the parser/Layer-B boundary,
   even though the tag is locally computable. The sharpened principle: **the
   parser surfaces form, the semantic layer assigns relational meaning.**
2. **Relations are PcsItems, not sibling blocks.** A PCS must stay one
   contiguous block, but relations can appear interspersed (a premise can carry
   a relation before its inference line). Emitting them as document-level
   blocks would fragment the PCS, so an interspersed relation is a
   `PcsItem::Relation`. It is still **flat** — a sibling in the item sequence,
   not nested under a statement node — reusing the A2b `Relation` (which carries
   its own indent); Layer B associates it to the preceding statement exactly as
   it does at document level.
3. **Reuse the statement parser for numbered-statement content** verbatim, so
   `(1) [P]: text`, `(1) [P]`, and `(1) plain` all parse (including the strict
   text-after-reference error). Reuse the A2b `relation` parser for relation
   items.
4. **Capture the literal number** (`usize`) from `(n)`, lossless; Layer B may
   renumber. Inference rule names are captured as `Vec<String>` (comma-split,
   trimmed); a bare divider yields an empty vec.
5. **Metadata stays raw / deferred.** A3 does not recognize `{…}` metadata,
   consistent with A2a leaving statement metadata in raw text. A5 will split
   metadata from inference rule names and statement text uniformly.

## AST (`argdown-core`) — additive

```rust
pub enum Block {
    Heading(Heading),
    Statement(Statement),
    Argument(Argument),
    Relation(Relation),
    Pcs(Pcs),
}

pub struct Pcs {
    pub items: Vec<PcsItem>,
    pub span: Span,            // first item span start → last item span end
}

pub enum PcsItem {
    /// `(n) <statement>` — content reuses the A2a statement forms.
    Statement {
        number: usize,
        statement: Statement,
        span: Span,           // the `(` of the marker → statement content end
    },
    /// `----` (bare → empty rules) or `-- Rule, Rule --` (ruled).
    Inference { rules: Vec<String>, span: Span },
    /// An interspersed relation line, reusing the A2b `Relation` (with indent).
    Relation(Relation),
}
```

All new types derive `Debug, Clone, PartialEq, Eq`. `Statement`, `Argument`,
and `Relation` are unchanged, so existing test literals do not break — only
`Block` gains a variant. Every item carries a byte span: `Statement` covers the
`(n)` marker through its content, `Inference` covers the divider line, and
`Relation` reuses the A2b `Relation` span (operator → target). `Pcs.span` runs
from the first item's span start to the last item's span end.

## Inference-line grammar

An inference line, after optional indentation, is one of:

| Form | Match | `rules` |
|---|---|---|
| Bare | `-{4,}` then end of line | `[]` |
| Ruled | `--` `<content>` `--` then end of line | content split on `,`, each trimmed, empties dropped |

Bare is tried before ruled, so `-----` is a bare divider (not `--` + `-` + `--`).
`---` matches neither (only three hyphens: too few for bare, no closing `--` for
ruled) and is a parse error. `<content>` is any text up to the closing `--`;
A3 captures it as rule names and does not interpret a `{…}` metadata block.

## Dispatch & the PCS item loop

`block` becomes `alt((heading, relation, pcs, argument, statement))`. `pcs` is
tried before `statement` because a `(n)` line would otherwise be plain text;
`pcs` succeeds only when the line begins (after indent) with a numbered marker
`( digits )`, otherwise it backtracks so `argument`/`statement` can run.

`pcs` parses a leading numbered statement, then loops, consuming one item per
iteration:

1. **numbered statement** — line begins with `( digits )`; parse the number,
   then reuse `statement` for the content.
2. **inference line** — bare (`-{4,}`) or ruled (`-- … --`). Tried **before**
   relation, and unambiguous against it: `- [X]` and `-> [X]` match neither the
   bare nor the ruled form, so they fall through.
3. **relation** — reuse the A2b `relation` parser (operator + target + indent).

The loop stops — ending the PCS block — at a blank line, a heading, a comment, a
top-level statement/argument, or EOF. Once a line commits to the PCS (a valid
`( digits )` marker), a malformed item is a hard error rather than a silent
block boundary.

## Continuation guards (`text.rs`)

A numbered statement's content may span continuation lines (reusing
`definition_body`/`content_line`). Those continuation lines must stop before the
next PCS item. The existing `at_content_line` guard already stops at relation
markers (first char `+`/`-`/`_`/`>` or `<`+sign), which covers **both** relation
items and inference lines (every inference line starts with `-`). A3 adds one
guard for the remaining case — a numbered marker — so a continuation line does
not swallow the next premise:

- `pcs_marker`: a line whose first non-space run is `( digits )`. Added to the
  `at_content_line` guard family next to `relation_marker`.

## Error model

Unchanged: `Err { message, offset }`, strict fail-fast. PCS-specific errors:

- `( digits )` with no parseable statement content → error (from the reused
  statement parser, under `cut_err` once the PCS is committed).
- A ruled inference opener `--` with no closing `--` on the line → error
  (matches the reference's "end your inference with two hyphens").
- Statement/relation target errors (e.g. text after a reference) are produced
  by the reused parsers unchanged.

## A1/A2a/A2b impact

- Additive `Block::Pcs` variant + new `Pcs`/`PcsItem` types; `Statement`,
  `Argument`, `Relation` unchanged → no existing test-literal churn.
- `block` gains a `pcs` dispatch arm before `statement`. The new arm only fires
  on a `( digits )` line, so prior dispatch behavior is unchanged.
- `at_content_line` gains the `pcs_marker` guard; the relation-marker guard
  already added in A2b covers inference lines and relation items.
- `argdown-mcp` unaffected (Debug-prints `Document`).

## Parser structure

- New `crates/argdown-parser/src/pcs.rs`: `pcs` (the block + item loop),
  `numbered_statement` (`( digits )` + reuse `statement`), `inference_line`
  (bare | ruled), and the item dispatch.
- `text.rs`: `at_content_line` gains the `pcs_marker` guard; add a `pcs_marker`
  peek next to `relation_marker`.
- `lib.rs`: `mod pcs;` + the `pcs` dispatch arm.

## Testing

Table-driven over `(input → expected Document)`:

- **Single-step PCS:** `(1) a\n(2) b\n----\n(3) c` → one `Block::Pcs` with four
  items in order: three `Statement` items (numbers 1, 2, 3) and one `Inference`
  with empty `rules`, between items 2 and 3.
- **Rule names:** `-- Modus Ponens --` → `rules == ["Modus Ponens"]`;
  `-- Rule A, Rule B --` → `["Rule A", "Rule B"]`.
- **Bare-divider dash counts:** `----` and `-----` both yield an `Inference`
  with empty `rules`; `---` is an error.
- **Multi-step / interleaved:** `(1) a\n(2) b\n----\n(3) c\n(4) d\n-- R --\n(5) e`
  → items in source order with two `Inference` items at the right positions.
- **Numbered-statement targets reuse statement forms:** `(1) [P]: text`,
  `(1) [P]`, `(1) plain`.
- **Interspersed child relation:** `(1) a\n  +> [X]\n----\n(2) b` → a
  `Relation` item (Support/Outbound, indent 2) between the first statement and
  the inference.
- **Multi-line numbered statement:** `(1) one\n    two` → statement text
  "one two".
- **Boundary:** a blank line, a heading, or a top-level `[statement]` ends the
  PCS (the following content is its own block).
- **Errors:** `(1)` with no content; a ruled inference opener with no closing
  `--`.
- **Regression:** all A1/A2a/A2b tests still pass.

## Success criteria

- `cargo test` passes, including the new PCS tests and all prior tests.
- `parse()` emits `Block::Pcs` items with correct numbers, inference rule names,
  relation items, and spans, in source order; numbered-statement content reuses
  the statement forms and relations reuse the A2b form.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
