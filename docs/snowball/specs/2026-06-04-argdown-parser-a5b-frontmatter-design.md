# Argdown Parser — Increment A5b (Document Frontmatter) — Design

- **Date:** 2026-06-04
- **Status:** Approved
- **Scope:** Recognize a single document-level YAML frontmatter block fenced by
  `===` lines at the start of the document. Capture the block's raw inner
  content + source span as `Metadata { raw, span }` on `Document.frontmatter`.
  YAML parsing is deferred (Layer B), exactly as A5a. This is the sibling
  increment to A5a (element `{…}` metadata).

## Context: the roadmap

A (parser) shipped: A1 (spine), A2a (arguments + references), A2b (relations),
A3 (PCS), A4 (inline), A5a (element `{…}` metadata). A5 (metadata) was split —
A5a (element metadata) then **A5b (this spec, document frontmatter)**. Later: B
semantic model, C JSON/Dung outputs, D MCP server.

The A5 metadata-representation decision already anticipated this increment:
`Metadata { raw, span }` is reused "on each site (statement/argument/heading/
inference) **+ Document.frontmatter**". A5b adds the `Document.frontmatter` site;
no new representation is introduced.

## Reference behavior (probed)

Confirmed against `@argdown/core` via MCP `export_json` / `parse`:

- **Start-only.** `===\n…\n===\n\n[S]: claim` parses; the statement begins
  *after* the frontmatter. A frontmatter block appearing after real content is a
  hard error ("Invalid paragraph start").
- **Leading trivia is fine.** A `// comment` (and blank lines) before the
  opening fence are accepted; the frontmatter is still recognized.
- **Fence leniency.** `====` (4+ `=`) is accepted, and an indented fence
  (`  ===`) is accepted.
- **Content is not YAML-validated at recognition time.** A malformed-YAML body
  still tokenizes as a frontmatter block (argdown parses the YAML in a later
  pass). A5b mirrors this: capture raw, defer the parse.
- **Trailing paragraph break.** argdown demands an empty line after the closing
  fence (it even rejects a frontmatter-only document that lacks one — a
  tokenizer quirk). A5b requires a blank line **or EOF** after the close (see
  decision 4).

## Decisions

All four strict/lenient knobs were settled with the operator
(`ask-user-question`); A5b leans toward argdown alignment and strict fail-fast,
consistent with the parser's established philosophy.

1. **Representation — reuse `Metadata { raw, span }` (pre-settled).**
   `Document` gains `frontmatter: Option<Metadata>`. `raw` is the verbatim YAML
   body between the fences (internal line endings preserved, both fence lines
   excluded); `span` runs from the first `=` of the opening fence to just past
   the last `=` of the closing fence (fences included; leading indent and the
   trailing newline excluded). **No YAML parse, no `serde_yaml`** — a single
   downstream Layer-B utility interprets `raw` once.
2. **Fence grammar — lenient mirror (D1).** A *fence line* is: optional leading
   whitespace, a run of **three or more** `=`, optional trailing whitespace,
   then a line ending or EOF. Opening and closing fences are matched
   independently (their `=` counts need not match). A line that is not purely a
   fence — `=== x`, `==`, `===text` — is **not** a fence line and is ordinary
   content.
3. **Non-leading fence — hard error (D2).** Frontmatter is recognized only as
   the document's first block (after leading blank lines / comments). A fence
   line appearing anywhere else is a hard `Err{message, offset}`. Two mechanisms
   enforce this: continuation readers stop at a fence line (they never absorb it
   as prose), and the block dispatcher rejects a fence line at a block boundary.
4. **Trailing paragraph break — blank line or EOF (D3).** After the closing
   fence, the next line must be blank, or the input must be at EOF; content
   beginning immediately on the line after the closing fence is a hard error.
   (Slightly more lenient than argdown, which rejects bare-EOF-without-blank;
   accepting EOF avoids rejecting a frontmatter-only document.)
5. **Unterminated frontmatter — hard error (D4).** An opening fence at document
   start with no closing fence before EOF is a hard error, consistent with
   A5a's unterminated-`{` rule.
6. **Deferred:** YAML parsing, the `parse_metadata` Layer-B utility (shared with
   A5a's element metadata), `tags`-key promotion, and any frontmatter→element
   inheritance. A5b only recognizes and locates the block.

## AST (`argdown-core`) — additive

```rust
/// A parsed Argdown document: optional frontmatter plus a flat block sequence.
pub struct Document {
    pub blocks: Vec<Block>,
    pub frontmatter: Option<Metadata>, // NEW — the `===…===` block, if present
}
```

`Metadata` is unchanged (`{ raw: String, span: Span }`, derives
`Debug, Clone, PartialEq, Eq`). `Document` already derives `Default`, so
`frontmatter` defaults to `None`; the empty-input and `Document::default()`
tests are unaffected. Existing `Document { blocks: … }` literals (one in
`ast.rs` tests, three in `lib.rs` tests, and the `document()` constructor) gain
`frontmatter: None` / `frontmatter` — the accepted additive churn.

## Recognition & wiring

New module `crates/argdown-parser/src/frontmatter.rs`:

- **`fence_marker`** — the single source of truth for a fence line. Consumes
  optional indent, captures the `={3,}` run's span via `with_span`, consumes
  optional trailing whitespace, then matches `line_ending | eof`. It is
  **backtrackable**: fewer than three `=`, or trailing non-whitespace, fails so
  the line is treated as ordinary content.
- **`fence_line`** = `fence_marker.void()`, exported `pub(crate)` for the two D2
  hooks.
- **`frontmatter`** — returns `Metadata`:
  1. Match the opening `fence_marker` (backtrackable, so `opt(frontmatter)` on a
     non-frontmatter document cleanly yields `None`). Record the `=`-run start.
  2. Read the raw body: `repeat(0.., body).with_taken()`, where `body` matches
     `not(fence_line)`, `not(eof)`, then one raw line (`till_line_ending` +
     `opt(line_ending)`). The taken slice is `raw` (verbatim, endings
     preserved). The repeat stops at the closing fence (body backtracks) or at
     EOF (body backtracks).
  3. Match the closing `fence_marker` under `cut_err`: reaching it at EOF means
     no closing fence was found → **unterminated error (D4)**. Record the
     `=`-run end.
  4. Under `cut_err`, require `peek(eof | blank_line | (inline_ws, eof))` →
     **trailing paragraph break (D3)**; otherwise error.
  5. Return `Metadata { raw, span: { start: open.start, end: close.end } }`.

`lib.rs` changes:

```rust
fn document(input: &mut Input<'_>) -> ModalResult<Document> {
    skip_trivia(input)?;                      // leading blank lines / comments
    let frontmatter = opt(frontmatter).parse_next(input)?;
    skip_trivia(input)?;                       // the blank line after the close
    let blocks = repeat(0.., terminated(block, skip_trivia)).parse_next(input)?;
    Ok(Document { blocks, frontmatter })
}
```

- **`block()`** gains a leading `misplaced_fence` alternative: `peek(fence_line)`
  → on match, return `ErrMode::Cut` (label: "frontmatter fence not at document
  start"); on non-match it backtracks so the existing heading/relation/pcs/
  argument/statement alternatives run unchanged.
- **`text.rs::at_content_line`** gains `not(fence_line)` so every continuation
  reader (statement, argument, PCS statement, relation target) stops at a fence
  line; the stray fence then surfaces as the `misplaced_fence` error at the next
  block boundary.

## Error model

Unchanged shape: `Err { message, offset }`, strict fail-fast. New errors:

- **Unterminated frontmatter:** opening fence at start, no closing fence before
  EOF (D4).
- **Missing paragraph break:** content on the line immediately after the closing
  fence (D3).
- **Misplaced fence:** a fence line anywhere after the document's first block
  (D2).

## A1–A5a impact

- Additive `frontmatter` field on `Document` → the `document()` constructor and
  four test literals gain `frontmatter`. No behavior change for any block type.
- `at_content_line` gains one `not(fence_line)` guard; because a pure fence line
  was never valid block content before, this only changes the
  previously-undefined `===`-as-text case (now a hard error per D2).
- `block()` gains the `misplaced_fence` cut-branch (first alternative).
- `Metadata`, `Block`, `Statement`, `Argument`, `Heading`, `Relation`, `Pcs`,
  `Inline` are untouched. `argdown-mcp` Debug-prints `Document`, unaffected.

## Parser structure

- New `crates/argdown-parser/src/frontmatter.rs`: `fence_marker`, `fence_line`
  (exported), `frontmatter`.
- `lib.rs`: register `mod frontmatter;`, wire `frontmatter` into `document()`,
  add the `misplaced_fence` branch to `block()`.
- `text.rs`: add `not(fence_line)` to `at_content_line`.
- `argdown-core`: add `Document.frontmatter`.

## Testing

Table-driven, mirroring the A5a metadata tests:

- **Basic:** `===\ntitle: X\nauthor: Y\n===\n\n[S]: claim` → `frontmatter`
  `raw == "title: X\nauthor: Y\n"`, one statement block; `&src[span] ==
  "===\ntitle: X\nauthor: Y\n==="`.
- **Absolute span:** a leading blank line before the fence shifts the span;
  `&src[span.start..span.end]` still slices the `===…===` block exactly.
- **Leading trivia:** a `// comment` and/or blank lines before the opening fence
  are accepted and the frontmatter is still recognized.
- **Fence leniency (D1):** `====` and an indented `  ===` open/close
  frontmatter; `=== x`, `===text`, and `==` are plain statements, not fences.
- **Empty body:** `===\n===\n` → `raw == ""`, frontmatter present.
- **CRLF:** `===\r\ntitle: X\r\n===\r\n` preserves `\r\n` in `raw` and the span
  slices correctly.
- **Non-YAML body:** a malformed-YAML body is still captured verbatim (defer).
- **No frontmatter:** a document with no opening fence → `frontmatter == None`;
  empty input → `Document::default()`.
- **Trailing break (D3):** `===\n…\n===\n\n[S]` ok; `===\n…\n===\n[S]` (no blank
  line) errors; `===\n…\n===` at EOF ok.
- **Unterminated (D4):** `===\ntitle: X\n[S]: claim` (no closing fence) errors.
- **Misplaced (D2):** `[S]: x\n\n===\n…\n===` errors; a bare `===` after content
  errors.
- **Regression:** all A1–A5a tests pass after the `frontmatter` field churn.

## Success criteria

- `cargo test` passes, including the new frontmatter tests and all prior tests.
- `parse()` populates `Document.frontmatter` with the correct raw content and
  absolute source span when a leading `===…===` block is present, and leaves it
  `None` otherwise; the unterminated, missing-paragraph-break, and misplaced-
  fence errors fire; fence leniency (D1) holds.
- Clean under `cargo clippy -- -D warnings`; canonical `cargo fmt`.
