# Argdown Parser — Increment A5a (Element Metadata) — Design

- **Date:** 2026-06-03
- **Status:** Approved
- **Scope:** Recognize trailing `{yaml}` metadata blocks on statements, argument
  descriptions, headings, and inference lines. Capture each block's raw inner
  content + source span as `Metadata { raw, span }`, stripped from the element's
  text. YAML parsing, a `parse_metadata` utility, tag-promotion, and document
  frontmatter are deferred. Frontmatter (`===` fences) is the sibling increment
  A5b.

## Context: the roadmap

A (parser) shipped: A1 (spine), A2a (arguments + references), A2b (relations),
A3 (PCS), A4 (inline). A5 (metadata) was split — **A5a (this spec, element
`{…}` metadata)** then A5b (document frontmatter). Later: B semantic model, C
JSON/Dung outputs, D MCP server.

## Reference behavior (probed)

Confirmed against `@argdown/core` via MCP `export_json`:

- `[S]: text {key: value}` → the `{…}` is consumed and the statement text is
  `"text "` (metadata **stripped** from text). The block attaches to the element
  it trails.
- **`{` is reserved.** `[Set]: the set {a, b} is large …` is an *error* — the
  first `{` opens metadata, and text after the closing `}` ("is large …") is
  rejected. So a `{` always opens metadata; metadata must be the **last** thing
  on the element (text-after-metadata is a hard error, like text-after-
  reference).
- **Multi-line blocks work.** `[S]: text {\n  certainty: 0.8\n  source: "a
  book"\n}` parses with text `"text "`; the body extends across the brace block
  to the closing `}`.
- **Inference lines** separate metadata from rule names: `-- Modus Ponens {uses:
  [1,2]} --` → `inferenceRules: ["Modus Ponens"]` (clean), with the `{…}` taken
  as the inference's metadata. This **fixes the A3 limitation** where the `{…}`
  polluted the captured rule name.

## Decisions

1. **Representation B — raw string + span, stripped (chosen via an M2 brain-
   jam over span-only and parse-now).** Each site gains `metadata:
   Option<Metadata>` where `Metadata { raw: String, span: Span }`: `raw` is the
   block's inner content verbatim, `span` is the whole `{…}` block's source
   range. The block is stripped from the element's `text`/`description`. **No
   YAML parse and no `serde_yaml` dependency** in the parser. The jam's key
   distinction: deferral fits when *interpretation varies by consumer* (A4
   markup); a YAML parse is *universal*, so deferring a bare span to N consumers
   distributes identical liability — capturing the raw string lets a single
   downstream `parse_metadata` utility interpret it once. Metadata is *typed
   data* (strip from text), not *prose markup* (which A4 kept as an overlay), so
   the strip-vs-retain asymmetry with A4 is intentional.
2. **`{` is a reserved opener (strict, reference-aligned).** An unescaped `{` in
   an element body opens metadata; `\{` is a literal brace (reuses A4's escape
   concept at the body-reader level). After the matching `}`, only trivia
   (whitespace, a trailing `// comment`, end of line) may follow; any other text
   is a hard `Err{message, offset}`.
3. **Balanced, multi-line, quote-aware matching.** The block runs from `{` to
   its matching `}`, tracking brace depth and skipping over `[…]` and quoted
   strings (`"…"`, `'…'`) so braces inside them don't miscount. The block may
   span multiple source lines; those lines are part of the element regardless of
   how they would otherwise scan (blank line, marker). An unterminated block (no
   matching `}` before EOF) is an error.
4. **Inference metadata splits cleanly.** A3's `ruled_divider` captured
   everything between `--` … `--` as comma-split rule names; A5a extracts a
   trailing `{…}` as the inference's `metadata` and leaves clean rule names.
5. **Relations need no metadata field.** `+ [B] {k: v}` attaches the metadata to
   the target statement/argument (recognized via the reused parsers), so the
   relation itself carries none.
6. **Deferred:** YAML parsing, the `parse_metadata(&str) -> Result<…,
   MetadataError>` utility (a single canonical Layer-B function with a crate-
   owned error enum), `tags`-key promotion, and document frontmatter (A5b).

## AST (`argdown-core`) — additive

```rust
/// A trailing `{yaml}` metadata block: raw inner content + source span. The
/// content is not YAML-parsed here (a Layer-B utility does that).
pub struct Metadata {
    pub raw: String,   // inner content, verbatim (between `{` and `}`)
    pub span: Span,    // the whole `{…}` block, source range
}

pub struct Statement { /* …existing… */ pub metadata: Option<Metadata> }
pub struct Argument  { /* …existing… */ pub metadata: Option<Metadata> }
pub struct Heading   { /* …existing… */ pub metadata: Option<Metadata> }

pub enum PcsItem {
    Statement { number: usize, statement: Statement, span: Span },
    Inference { rules: Vec<String>, metadata: Option<Metadata>, span: Span }, // + metadata
    Relation(Relation),
}
```

`Metadata` derives `Debug, Clone, PartialEq, Eq`. `Relation`, `Pcs`, `Document`,
and `Inline` are unchanged. Every existing `Statement`/`Argument`/`Heading`
literal gains `metadata: None` (the accepted churn, as with A4's `inlines`).

## Recognition & attachment

- A new `metadata` recognizer matches a balanced `{…}` block (decision 3) and
  returns `Metadata { raw, span }`. It is reused at every site.
- The metadata `{` is detected at the **body's top level**, interleaved with A4
  inline recognition: a `{` *inside* a recognized inline element (e.g. a link
  URL or display text, a mention) is part of that element and does not open
  metadata. The metadata opener is the first unescaped top-level `{`.
- **Statement / argument definition bodies:** the body text is read up to an
  unescaped top-level `{`; the metadata block is then consumed (possibly multi-
  line); then end-of-element is required (trivia only — else error). The text
  excludes the metadata; inline recognition (A4) runs on the text portion only.
- **Plain statements:** same — body text up to a `{`, then metadata, then strict
  end.
- **Headings:** after the heading text, an optional `{…}` metadata block; strict
  end.
- **Inference lines (PCS):** the `--` … `--` content is split into comma-
  separated rule names and an optional trailing `{…}` metadata block.
- A statement/argument **reference** (`[T]`, `<T>`) carries `metadata: None`
  unless a `{…}` trails it; `[T] {k: v}` attaches metadata to the reference.

## Error model

Unchanged shape: `Err { message, offset }`, strict fail-fast. New metadata
errors: text after a closing `}` on an element (text-after-metadata); an
unterminated `{` with no matching `}` before EOF. A `\{` is literal and never
opens a block.

## A1–A4 impact

- Additive `metadata` field on `Statement`/`Argument`/`Heading` and
  `PcsItem::Inference` → every existing literal for those gains `metadata: None`
  (mechanical churn). `Relation`/`Pcs`/`Document`/`Inline` untouched.
- The shared body readers (`definition_body`, `plain_statement`) gain metadata
  splitting; PCS statements and relation targets inherit it via reuse.
- A4 inline recognition now runs on the pre-metadata text only — a `{` ends the
  inline-scanned region. A4's `\` escape extends to `\{`.
- The A3 inference `ruled_divider` is reworked to separate rule names from
  metadata (a behavior fix: rule names are now clean).
- `argdown-mcp` unaffected (Debug-prints `Document`).

## Parser structure

- New `crates/argdown-parser/src/metadata.rs`: the balanced-block recognizer
  (depth/quote/escape aware, multi-line) returning `Metadata`.
- `text.rs`: body readers split text from a trailing metadata block and enforce
  the strict text-after rule.
- `statement.rs` / `argument.rs` / `heading.rs`: attach `metadata`.
- `pcs.rs`: inference line splits rule names from metadata.
- `argdown-core`: `Metadata` type; re-export.

## Testing

Table-driven over `(input → expected metadata)`:

- **Statement / argument / heading single-line:** `[S]: text {k: v}` →
  `text == "text"`, `metadata.raw == "k: v"`; `<A>: d {k: v}`; `# H {k: v}`.
- **Inference:** `-- Modus Ponens {uses: [1,2]} --` → `rules == ["Modus
  Ponens"]`, `metadata.raw == "uses: [1,2]"`.
- **Multi-line block:** `[S]: text {\n a: b\n c: d\n}` → text `"text"`,
  `metadata.raw` contains both lines; body span extends to the `}`.
- **Balanced / quote-aware:** `{outer: {inner: 1}}` and `{note: "a } b"}` capture
  the full block (inner/quoted braces don't terminate early).
- **Strip + inline coexist:** `[S]: a **bold** claim {k: v}` → text
  `"a **bold** claim"`, one Bold inline, plus metadata.
- **Escape:** `[S]: a \{ literal brace` → no metadata, `{` is literal text.
- **Strict errors:** `[S]: the set {a} more` (text after `}`) → error;
  `[S]: text {a: b` (unterminated) → error.
- **Reference with metadata:** `[T] {k: v}` → reference statement with metadata.
- **Regression:** all A1–A4 tests pass after the `metadata: None` churn.

## Success criteria

- `cargo test` passes, including new metadata tests and all prior tests.
- `parse()` populates `metadata` on statements, arguments, headings, and
  inference lines with the correct raw content and source span, stripped from
  the element text; inference rule names are clean; the strict text-after and
  unterminated-block errors fire; `\{` is literal.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
