# Argdown Parser — Increment A2b (Relations) — Design

- **Date:** 2026-06-02
- **Status:** Approved
- **Scope:** Parse relation lines (all 10 operators) and their targets, as a
  flat sequence of blocks tagged with indentation depth. Tree assembly is
  deferred to the semantic model (Layer B).

## Context: the roadmap

A2 (relational core) was split into A2a (arguments + references, shipped) and
A2b (relations, this spec). Earlier: A1 (spine). Later: A3 PCS, A4 inline, A5
metadata; then B semantic model, C JSON/Dung outputs, D MCP server.

## Reference behavior (probed)

Relation direction, confirmed against `@argdown/core` via MCP `export_json`:

| Symbols | type | edge (from → to) |
|---|---|---|
| `+`, `<+` | support | target → parent |
| `+>` | support | parent → target |
| `-`, `<-` | attack | target → parent |
| `->` | attack | parent → target |
| `_`, `<_` | undercut | target → parent |
| `_>` | undercut | parent → target |
| `><` | contradictory | symmetric (recorded parent → target) |

Nesting is by indentation: a more-indented relation is a child of the nearest
preceding less-indented line; siblings share indentation. A relation's target
may be a statement or an argument (ref/def), and that target may itself have
deeper child relations (recursion). The `support`-vs-`entails` /
`attack`-vs-`contrary` distinction is a mode-dependent semantic resolution —
Layer B, not parsing.

## Decisions

1. **Flat-with-depth representation.** The parser emits relation lines as flat
   `Block::Relation` entries in source order, each carrying its raw
   indentation. Assembling the parent/child tree and the relation graph is
   Layer B's job. This keeps the parser a pure line recognizer (consistent
   with A1's flat block list) and requires **no changes to the `Statement` or
   `Argument` structs**.
2. **Collapse `+` ≡ `<+`** (and `-` ≡ `<-`, `_` ≡ `<_`). They are semantically
   identical; the reference produces identical edges. The parser records
   `(operator, direction)`, not the raw token.
3. **Direction is expressed relative to the implicit parent**
   (`Inbound`/`Outbound`/`Bidirectional`). The flat parser does not resolve the
   parent; it records the arrow's meaning and Layer B applies it.
4. **Indentation is captured as a raw leading-whitespace count** (`usize`),
   lossless; Layer B normalizes to logical depth.

## AST (`argdown-core`) — additive

```rust
pub enum Block {
    Heading(Heading),
    Statement(Statement),
    Argument(Argument),
    Relation(Relation),
}

pub struct Relation {
    pub indent: usize,            // count of leading whitespace chars before the operator
    pub operator: RelationOperator,
    pub direction: RelationDirection,
    pub target: RelationTarget,
    pub span: Span,               // operator start → target end (excludes the indent)
}

pub enum RelationOperator { Support, Attack, Undercut, Contradictory }

/// Direction relative to the implicit parent element (the less-indented line above).
/// `Inbound`  = relation points from the target to the parent (`+`, `<+`, etc.).
/// `Outbound` = from the parent to the target (`+>`, `->`, `_>`).
/// `Bidirectional` = `><`.
pub enum RelationDirection { Inbound, Outbound, Bidirectional }

pub enum RelationTarget { Statement(Statement), Argument(Argument) }
```

All new types derive `Debug, Clone, PartialEq, Eq`. `Statement`/`Argument` are
unchanged, so existing test literals do not break — only `Block` gains a
variant.

## Operator mapping

| Token | operator | direction |
|---|---|---|
| `+`, `<+` | Support | Inbound |
| `+>` | Support | Outbound |
| `-`, `<-` | Attack | Inbound |
| `->` | Attack | Outbound |
| `_`, `<_` | Undercut | Inbound |
| `_>` | Undercut | Outbound |
| `><` | Contradictory | Bidirectional |

Operator tokens are matched two-char-before-one-char (`+>` before `+`,
`<+` before any `+`-form) so prefixes don't shadow longer tokens.

## Indentation capture (the subtle part)

Today `skip_trivia` greedily consumes *all* whitespace, including the next
line's leading indent — which relations need. A2b **reworks `skip_trivia` to be
line-structured**: it consumes blank lines and comment lines (each possibly
indented) and bare line endings, but stops at the leading indent of a
content-bearing line, leaving that indent for the block parser to measure.

Concretely, between blocks it repeats `alt(( (inline_ws, line_ending),
(inline_ws, comment) ))`: a blank line or a comment line is consumed whole; a
content line makes both branches fail at/after `inline_ws`, and `alt`
backtracks (restoring the consumed indent), so the cursor rests at column 0 of
the content line. Top-level elements sit at indent 0, so heading/statement/
argument parsing is unchanged. `relation` measures `inline_ws` to get `indent`,
then parses the operator and target.

## Dispatch & target reuse

`block` tries, in order: `heading`, `relation`, `argument`, `statement`.

- `relation` matches an operator token after the indent. Its `<+`/`<-`/`<_`
  forms match `<` followed by `+`/`-`/`_`; a `<Title>` argument does not match
  (the char after `<` is a title char), so it falls through to `argument`.
- The relation **target reuses the A2a `argument` and `statement` parsers
  verbatim**: `relation_target = alt((argument → Argument, statement →
  Statement))`. So `+ [B]`, `+ [B]: x`, `+ plain`, `+ <Arg>`, `+ <Arg>: x` all
  parse, including the strict text-after-reference error (`+ [B] extra` →
  `Err`).

One guard change in `text.rs`: `at_content_line` gains a "not a relation
marker" check (a line whose first non-space char is `+`/`-`/`_`/`>`, or `<`
followed by `+`/`-`/`_`). This (a) stops a multi-line target's text from
swallowing a following relation line, and (b) makes `[A]\n  + [B]` parse as a
statement reference followed by a child relation block — not a
text-after-reference error.

## Error model

Unchanged: `Err { message, offset }`, strict fail-fast. Relation-target errors
are produced by the reused statement/argument parsers (e.g. text after a
reference target). A relation operator with no parseable target (e.g. `+ ` at
end of input) fails to parse and surfaces as a parse error.

## A2a/A1 impact

- Additive `Block::Relation` variant + new relation types; `Statement`/
  `Argument` unchanged → no existing test-literal churn.
- `skip_trivia` rework is the one regression-risk area; the existing A1/A2a
  tests (multi-line statements, blank-line separation, comments, headings) are
  the safety net and must stay green.
- `at_content_line` gains one guard; verified by re-running the A2a
  reference/error tests plus new relation-under-reference tests.
- `argdown-mcp` unaffected (Debug-prints `Document`).

## Parser structure

- New `crates/argdown-parser/src/relation.rs`: `relation`, `relation_operator`
  (token → operator+direction), `relation_target`.
- `trivia.rs`: `skip_trivia` reworked to line-structured; add a `relation_marker`
  peek (or place it in `text.rs` next to the other guards).
- `text.rs`: `at_content_line` gains the relation-marker guard.
- `lib.rs`: `mod relation;` + the relation dispatch arm.

## Testing

Table-driven over `(input → expected Document)`:

- Each operator/direction maps correctly: `+`,`<+` (Support/Inbound), `+>`
  (Support/Outbound), `-`,`<-`,`->`, `_`,`<_`,`_>`, `><` (Contradictory/
  Bidirectional).
- Indent captured: `  + [B]` → `indent == 2`; `    - [C]` → `indent == 4`.
- Nested example `[A]\n  + [B]\n    - [C]\n  + [D]` → four blocks: the `[A]`
  statement plus three `Relation`s with indents 2, 4, 2 and the right
  operators/targets, in source order.
- Targets: statement reference (`+ [B]`), statement definition (`+ [B]: x`),
  plain (`+ claim`), argument reference (`+ <Arg>`), argument definition
  (`+ <Arg>: desc`).
- Relation under a reference: `[A]\n  + [B]` → reference block + relation block
  (no error).
- Multi-line relation target: `+ [B]: one\n    two` → target description
  "one two".
- Regression: all A1/A2a tests still pass after the `skip_trivia` rework.

## Success criteria

- `cargo test` passes, including the new relation tests and all prior tests.
- `parse()` emits `Block::Relation` entries with correct operator, direction,
  indent, target, and span; targets reuse the statement/argument forms.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
