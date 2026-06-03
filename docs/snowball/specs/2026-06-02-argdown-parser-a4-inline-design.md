# Argdown Parser — Increment A4 (Inline Elements) — Design

- **Date:** 2026-06-02
- **Status:** Approved
- **Scope:** Recognize inline elements inside statement text and argument
  descriptions — bold, italic, link, statement-mention, argument-mention, tag —
  as a flat list of typed source-span `Inline`s overlaid on the source. The
  `text`/`description` strings are unchanged (markup stays literal). Mention/tag
  *resolution* and aggregation, and any display-text cleaning, stay downstream;
  `{yaml}` metadata stays A5.

## Context: the roadmap

A (parser) shipped: A1 (spine), A2a (arguments + references), A2b (relations),
A3 (PCS). This is **A4 (inline)** — the first increment that enriches the text
*body* of existing blocks rather than adding a new block kind. Later: A5
metadata; then B semantic model, C JSON/Dung outputs, D MCP server.

## Reference behavior (probed)

Confirmed against `@argdown/core` via MCP `export_json`:

- Inline elements are represented as **`ranges`** — typed spans (`bold`,
  `italic`, `link`, `statement-mention`, `argument-mention`, `tag`) over the
  statement text. Each carries offsets plus element data (`url`, `title`,
  `tag`).
- **Nesting emits contained flat ranges**, not a tree: `**bold and *italic*
  inside**` yields a `bold` range that contains an `italic` range.
- **Strict on completion:** an unclosed `**…` is a hard error ("Incomplete bold
  text range… use `\` to escape any character"). Recognition is **context-
  gated**, so `5 * 3`, `#1`, and `a@b` do *not* error — a delimiter only opens
  markup when adjacent to non-whitespace.
- **Escaping:** `\` makes any character literal.
- **Tags** have two forms: `#contiguous-tag` (letters/digits/hyphens) and
  `#(multi word)` (parenthesized, spaces allowed).
- A bare `[Other]` mid-text is *not* an inline statement reference — inline
  references are `@[Other]`; bare brackets only mean a title/reference at the
  start of a statement (A2a).

The reference stores offsets into a *cleaned* text (markup stripped, and
inconsistently — `**`/links stripped, `@[]`/`#` kept). **A4 deliberately
diverges**: it keeps every span a true *source* range (below).

## Decisions

1. **Representation: flat source-span inlines (chosen via an M2 brain-jam over a
   reference-style cleaned-text model and a recursive node tree).** `Statement`
   and `Argument` gain `inlines: Vec<Inline>`, a flat list in source order; each
   `Inline` carries a source `Span`. `text`/`description` keep their normalized
   whitespace with **markup retained**. Rationale: every span stays a true
   source range (consistent with A1–A3 and required for diagnostics/tooling); a
   cleaned-text model breaks that back-mapping; a node tree maximizes churn and
   bakes in interpretation. **Nesting** is expressed by span containment, not
   recursion (mirrors the reference's contained ranges).
2. **Strict on completion (operator choice, reference-aligned).** An
   unambiguously *opened* element that doesn't complete is a hard `Err{message,
   offset}`: unclosed `**…`, `[text](` with no `)`, `@[`/`@<`/`#(` with no
   closer. This sits alongside the existing block-level strict fail-fast.
3. **Context-gated recognition keeps prose safe.** A delimiter opens markup only
   when adjacent to non-whitespace; `_`/`__` add a word-boundary guard. So `5 *
   3`, `snake_case`, `a@b`, and `[1]` stay literal and never error.
4. **Escaping.** `\x` makes `x` literal and suppresses markup at that position.
   The `\` stays in the raw `text`; stripping it for display is a deferred
   downstream concern (Decision 6).
5. **`inlines` is an independent source overlay, not an index into `text`.**
   Each `Inline.span` is an absolute *source* range. `text`/`description` stay
   normalized (whitespace collapsed) and are *not* verbatim source substrings,
   so the spans deliberately do not index into them — they index the original
   source. This keeps every span a true source range (the A1–A3 invariant)
   without changing how `text` is built.
6. **Display-text cleaning and containment indexing are deferred (not built in
   A4).** A `clean_text`/render helper is intentionally *out of scope*: because
   spans are source offsets while `text` is normalized, the helper must read
   source, and its contract (source-faithful vs normalized output) depends on
   the output layer (C), which doesn't exist yet — building it now would commit
   to a consumer-less contract. Likewise a containment/interval index waits for a
   real query consumer; it would be an external free function over `&[Inline]`,
   never a field on the node. A4 ships the canonical spans; consumers arrive in
   B/C.
7. **Churn is accepted here.** Adding a field to `Statement`/`Argument` means
   every existing literal gains `inlines: vec![]`. This is the first increment to
   enrich existing block bodies; `Heading` is untouched, and inline applies only
   to statement text and argument descriptions (headings stay raw).

## AST (`argdown-core`) — additive field

```rust
pub struct Statement {
    pub title: Option<String>,
    pub text: String,            // normalized whitespace, markup retained
    pub is_reference: bool,
    pub span: Span,
    pub inlines: Vec<Inline>,    // NEW: flat, source-order, over `text`
}

pub struct Argument {
    pub title: String,
    pub description: String,
    pub is_reference: bool,
    pub span: Span,
    pub inlines: Vec<Inline>,    // NEW: over `description`
}

/// One inline element. `span` is the full source extent of the element,
/// opening delimiter through closing delimiter.
pub struct Inline {
    pub kind: InlineKind,
    pub span: Span,
}

pub enum InlineKind {
    Bold,
    Italic,
    Link { url: String },
    StatementMention { title: String },
    ArgumentMention { title: String },
    Tag { tag: String },
}
```

`Inline`/`InlineKind` derive `Debug, Clone, PartialEq, Eq`; `Span` stays `Copy`.
A reference (`is_reference == true`) carries `inlines: vec![]` (its body is
empty). `Heading` is unchanged.

## Recognition grammar

Recognition scans the **body source** of a statement/argument left to right.
Each element has a commitment point; once committed, a missing closer is a hard
error.

| Element | Open / pattern | Data | Commit point |
|---|---|---|---|
| Bold | `**X**` / `__X__` | — | the run before non-ws (`_`: + word boundary) |
| Italic | `*X*` / `_X_` | — | the run before non-ws (`_`: + word boundary) |
| Link | `[text](url)` | `url` | the `(` after `[text]` |
| Statement-mention | `@[Title]` | `title` | `@[` |
| Argument-mention | `@<Title>` | `title` | `@<` |
| Tag | `#tag` or `#(multi word)` | `tag` | `#` before a tag char or `(` |

- **Emphasis flanking (simplified, not full CommonMark).** A delimiter run opens
  only when immediately followed by a non-whitespace, non-delimiter char, and
  closes only when immediately preceded by a non-whitespace char. For `_`/`__`,
  the char before an opener (and after a closer) must not be alphanumeric, so
  `snake_case` is literal. Bold is matched before italic.
- **Link.** `[` display `]` `(` url `)`. A `[…]` not immediately followed by `(`
  is literal (so `[1]` is prose). `display` is the source between `[` and `]`;
  `url` is the source between `(` and `)`.
- **Mentions.** `@[` title `]` or `@<` title `>`; `title` trimmed. A lone `@`
  not followed by `[`/`<` is literal.
- **Tag.** `#` then a contiguous run of `[A-Za-z0-9_-]` → that run; or `#(` then
  any chars to `)` → that string. A `#` not followed by a tag char or `(` (e.g.
  `# `) is literal.
- **Escape.** `\` consumes the next char as literal; no element is recognized at
  that position.
- Characters that never form a recognized opener are literal — no element, no
  error.

## Error model

Unchanged shape: `Err { message, offset }`, fail-fast. New inline errors fire at
the commitment point's element when it does not complete before the end of the
body: unclosed bold/italic run, `[text](` with no `)`, `@[`/`@<` with no closer,
`#(` with no `)`. Offsets point at the opener. Block-level errors are unchanged.

## Deferred (explicitly not in A4)

- **Display-text cleaning / `clean_text` helper** — deferred per Decision 6
  (spans are source offsets, `text` is normalized; the helper's contract depends
  on the unbuilt output layer). A4 ships canonical spans only.
- **Containment / interval index** — deferred until a real query consumer; an
  external free function over `&[Inline]`, never a node field.
- **Mention/tag resolution and tag aggregation** — Layer B.

## Parser structure

- New `crates/argdown-parser/src/inline.rs`: `inlines(body_source) ->
  Vec<Inline>` plus per-element recognizers and the escape/flanking helpers.
- `statement.rs` / `argument.rs`: after building the normalized body text, run
  inline recognition over the body's source and attach `inlines`. The normalized
  `text` is built as today, unchanged.
- `text.rs`: may host shared scanning helpers if reused across statement and
  argument.
- `argdown-core`: `Inline`/`InlineKind` types; re-export.

## A1/A2a/A2b/A3 impact

- `Statement`/`Argument` gain an `inlines` field → **every existing test literal
  for those types adds `inlines: vec![]`** (mechanical churn across lib.rs and
  ast.rs tests). `Heading`, `Relation`, `Pcs` are untouched.
- PCS numbered-statement targets and relation targets reuse `statement`, so they
  pick up `inlines` automatically — no separate wiring.
- `argdown-mcp` unaffected (Debug-prints `Document`).

## Testing

Table-driven over `(input → expected inlines)`:

- **Each emphasis form:** `*i*`, `_i_`, `**b**`, `__b__` → one Italic/Bold with
  the right source span.
- **Link:** `[text](http://x)` → `Link { url: "http://x" }` spanning the whole.
- **Mentions:** `@[S]` → `StatementMention { title: "S" }`; `@<A>` →
  `ArgumentMention { title: "A" }`.
- **Tags:** `#simple-tag` → `Tag { tag: "simple-tag" }`; `#(multi word)` →
  `Tag { tag: "multi word" }`.
- **Nesting:** `**bold and *italic* inside**` → Bold span containing an Italic
  span.
- **Prose stays literal (no error, no inlines):** `5 * 3`, `snake_case`, `a@b`,
  `[1]`, `cost #1`.
- **Escaping:** `\*not italic\*` → no Italic; text retains the raw chars.
- **Strict errors:** unclosed `**bold`, `[text](` with no `)`, `@[S` with no
  `]`, `#(tag` with no `)`.
- **Inline in argument descriptions** and **in PCS numbered statements** (via
  reuse).
- **Regression:** all A1–A3 tests pass after the `inlines: vec![]` literal churn.

## Success criteria

- `cargo test` passes, including new inline tests and all prior tests.
- `parse()` populates `Statement.inlines` / `Argument.inlines` with correct
  kinds, data, and source spans, in source order, with nesting by containment;
  prose with stray delimiters stays literal; unclosed recognized markup errors.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
