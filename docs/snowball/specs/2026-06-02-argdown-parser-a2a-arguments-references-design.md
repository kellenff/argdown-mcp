# Argdown Parser — Increment A2a (Arguments & References) — Design

- **Date:** 2026-06-02
- **Status:** Approved
- **Scope:** Add argument definitions/references and statement references as
  new top-level block kinds, with the reference's strict "no text after a
  reference" error. No relations yet.

## Context: the roadmap

The relational core (roadmap layer A2) is split into two increments:

- **A2a** *(this spec)*: arguments (`<Title>: desc`, `<Title>`) and statement
  references (`[Title]`) as new block kinds. No relations.
- **A2b**: nested relations (all operators/directions, indentation-driven
  recursion) between statements and arguments.

Earlier increments shipped: A1 (spine — headings, plain/titled statements,
comments). Later layers: A3 PCS, A4 inline, A5 metadata; then B semantic
model, C JSON/Dung outputs, D MCP server.

## Reference behavior (probed)

From the reference implementation (`@argdown/core` via MCP `export_json`):

- `<Arg>: This is the description.` → argument **definition**; description is
  the text after the colon, multi-line and normalized (wrapped lines joined
  with a single space), exactly like statements.
- `<Arg>` alone → argument **reference** (`isReference: true`, empty
  description).
- `[Foo]` alone → statement **reference**.
- `<Arg> trailing words` and `[Foo] trailing words` → **error**: "Invalid
  position of text content. Make sure it is not preceded by a statement
  reference or argument reference."
- `[Foo]\nbar` (reference, then an unseparated content line) → the **same
  error** at `L2:1`. A reference takes no continuation lines.

## Decisions

1. **Model references with an `is_reference` flag**, not separate types. This
   mirrors the reference model's `isReference` and composes cleanly when A2b
   makes statements and arguments relation targets.
2. **Match the reference's strict error**: free text after a reference (same
   line or an unseparated next line) is a hard `Err`. This reverses A1's
   lenient "bare bracket = plain text" rule.
3. **Block dispatch by first character class**: `#` → heading, `<` →
   argument, `[` → statement definition/reference, else → plain statement.

## AST changes (`argdown-core`)

```rust
pub struct Statement {
    pub title: Option<String>,   // None = plain text
    pub text: String,            // "" when is_reference
    pub is_reference: bool,      // true for `[Title]`
    pub span: Span,
}

pub struct Argument {
    pub title: String,           // arguments are always titled
    pub description: String,     // "" when is_reference
    pub is_reference: bool,      // true for `<Title>`
    pub span: Span,
}

pub enum Block {
    Heading(Heading),
    Statement(Statement),
    Argument(Argument),
}
```

`Statement` gains `is_reference`. `Argument` is new and lives in `ast.rs`
beside the others (re-exported from `lib.rs`). All derive
`Debug, Clone, PartialEq, Eq`.

Surface-form mapping:

| Source | AST |
|--------|-----|
| `plain text` | `Statement { title: None, text, is_reference: false }` |
| `[T]: x` | `Statement { title: Some("T"), text: "x", is_reference: false }` |
| `[T]` | `Statement { title: Some("T"), text: "", is_reference: true }` |
| `<T>: x` | `Argument { title: "T", description: "x", is_reference: false }` |
| `<T>` | `Argument { title: "T", description: "", is_reference: true }` |

## Grammar rules (precise)

- **Block dispatch** (after trivia): a heading (`#…`), else an argument
  (first non-space char `<`), else a statement (`[…]` or plain).
- **Argument definition** `<Title>: desc`: `<`, title chars up to `>`, `>`,
  `:`, optional space, then description — multi-line continuation and
  normalization identical to statement text (trim each line, join with a
  single space; trailing `//` line comment stripped).
- **Argument reference** `<Title>`: `<`, title, `>`, then only optional
  whitespace and/or a trailing line comment before the line ending or EOF.
- **Statement reference** `[Title]`: `[`, title, `]`, then only optional
  whitespace/trailing line comment. (`[Title]: x` remains a definition, from
  A1.)
- **Strict text-after-reference error:** once a reference (`[T]` or `<T>`) is
  recognized, the only things allowed after it are whitespace, a trailing
  line comment, a line ending followed by a block boundary (blank line, EOF,
  comment line, or a new block-start `#`/`[`/`<`). Free text — on the same
  line after the closing bracket, or on an immediately following
  non-structural line — is an `Err { message, offset }` at the offending
  byte. References take no continuation lines; definitions and plain
  statements still do.
- **Title contents:** the characters between the brackets, trimmed (A1's
  `[T]:` already trims the title; arguments follow the same rule). Titles may
  contain spaces.

## Error model

Unchanged from A1: `Err { message, offset }` (byte offset), strict
fail-fast. New error: text after a reference, with `offset` at the start of
the offending text.

## A1 impact

- The A1 test `bare_bracket_without_colon_is_plain_text` is **reversed**:
  `[Foo] is text` now produces an `Err` (text after a reference). The test is
  rewritten to assert the error.
- Every existing `Statement` literal (in `argdown-parser` tests and in the
  `statement` constructor) gains `is_reference: false`. Mechanical.
- `argdown-mcp` is unaffected (it `Debug`-prints `Document`).

## Parser structure (`argdown-parser`)

- New `argument.rs`: `argument` parser (definition + reference), reusing the
  trivia helpers and the normalization logic.
- `statement.rs`: add the reference path (`[T]` with no colon) and the
  text-after-reference error; share normalization with arguments.
- `lib.rs` block dispatch gains the `<…>` arm.
- A shared helper for "title in brackets" (`[..]` / `<..>`) and for the
  normalize-lines logic keeps `statement.rs` and `argument.rs` DRY.

## Testing

Table-driven over `(input → expected Document | Err)`:

- argument definition, single line: `<A>: desc`
- argument definition, multi-line: `<A>: line one\nline two`
- argument reference: `<A>`
- statement reference: `[S]`
- statement definition still works: `[S]: text`
- plain statement still works
- heading still works
- error: `[S] words` (offset at `words`)
- error: `<A> words` (offset at `words`)
- error: `[S]\nwords` (offset at `words` on line 2)
- error: `<A>\nwords`
- reversed A1 test: `[Foo] is text` → `Err`

## Success criteria

- `cargo test` passes, including the new A2a tests and the rewritten A1
  bracket test.
- `parse()` produces `Argument`/`Statement` reference and definition blocks
  for the surface forms above, and `Err { message, offset }` for
  text-after-reference.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
