# Argdown Parser — Increment A1 (Spine) — Design

- **Date:** 2026-06-02
- **Status:** Approved
- **Scope:** First real winnow grammar for `argdown-parser`: the document
  "spine" — headings, statements, comments — plus the `argdown-core` AST
  types they produce.

## Context: the roadmap

The full Argdown → MCP effort is decomposed into layers, each its own
spec → plan → build cycle:

- **A. Parsing** (`argdown-parser`, winnow) — the syntax tree, built
  construct-by-construct:
  - **A1 spine** *(this spec)*: sections/headings, statements (plain +
    titled), comments, blank-line block structure.
  - A2 relational core: arguments (defs + refs), relations (nested, all
    symbols), statement references.
  - A3 PCS: premise-conclusion structures, inference markers.
  - A4 inline: bold/italic, links, tags, mentions, shortcodes → ranges.
  - A5 metadata: frontmatter, inline/block `{data}`.
- **B. Semantic model** (`argdown-core` or a new `argdown-model`):
  equivalence classes, relation-graph resolution, argument/PCS assembly,
  section tree, tag registry, statement roles.
- **C. Outputs:** JSON export matching `@argdown/core`'s shape; Dung
  extensions in Rust.
- **D. MCP server** (`argdown-mcp`): wire A–C behind `parse` /
  `export_json` / `dung_extensions`.

This spec covers **A1 only**.

## Decisions (with rationale)

1. **AST shape: flat block sequence**, not a nested section tree. The
   parser stays a pure line/block recognizer; section nesting and
   statement→section assignment are deferred to Layer B.
2. **Spans: byte offsets baked into every node now.** Retrofitting
   positions later is invasive. Line/column is computed at the output
   boundary when needed.
3. **Robustness: strict fail-fast.** `parse()` returns
   `Result<Document, Error>` and stops at the first syntax error with a
   message + byte offset. Error-recovery (partial AST + diagnostics, like
   the reference) is a later increment.
4. **Text ownership: owned `String`.** Considered borrowed `Cow<'a, str>`
   and spans-only. Rationale (validated in an M2 brain-jam, transcript was
   under `.brainstorm/`):
   - The feared `String → Cow` migration is *not* the only optimization
     path. Because every node already carries a byte `Span`, a zero-copy
     escape hatch can be added **additively** later — `&source[span]`
     accessors are a *new* API, not a breaking type change.
   - At the MCP/JSON boundary the text must be serialized anyway, so
     spans-only buys nothing on the wire; it would only help in-process
     traversal, which is query-then-discard and not the bottleneck
     (parsing dominates over text access).
   - Spans-only / `Cow` only wins if Layer B does multi-pass analysis that
     retains text across passes under memory pressure — for which there is
     no evidence yet. Absent data, owned strings are the simple,
     self-contained, MCP-friendly default.
5. **Span anchored to original source; `text` holds normalized content.**
   `span` precisely locates the construct (including wrapping and stripped
   comments); `text` is the cleaned, joined/trimmed string. Thus
   `&source[span]` may differ from `text` — by design.

## Scope

**In:**
- ATX headings (`#`–`######`).
- Plain statements.
- Titled statements (`[Title]: text`).
- The three comment forms (`//`, `/* */`, `<!-- -->`) — recognized and
  discarded.
- Blank-line block separation.
- Multi-line wrapped statement text (normalized).
- Byte spans on every node.
- Strict fail-fast errors with a byte offset.

**Out (deferred to later increments):**
- Statement *references* (`[Title]` with no colon) — parsed as plain text
  in A1; reference semantics arrive in A2.
- Arguments, relations, PCS, inline formatting (statement text kept raw),
  tags, mentions, frontmatter / `{data}`.
- Equivalence-class and section-tree assembly (Layer B).
- Error recovery.

## AST (`argdown-core`)

```rust
pub struct Span { pub start: usize, pub end: usize } // byte offsets into source

pub struct Document { pub blocks: Vec<Block> }       // Default = empty

pub enum Block { Heading(Heading), Statement(Statement) }

pub struct Heading   { pub level: u8, pub text: String, pub span: Span }
pub struct Statement { pub title: Option<String>, pub text: String, pub span: Span }
```

- All derive `Debug, Clone, PartialEq, Eq`. `Document` also derives
  `Default`.
- `span` = original source range; `text` = normalized content.
- Module split: `ast.rs` (the types above), `error.rs` (the `Error` type),
  re-exported from `lib.rs`. Keeps files focused.

## Grammar rules (precise)

- **Blocks** are separated by one or more blank lines (lines that are empty
  or whitespace-only). A block's **first line** decides its kind.
- **Heading**: a line of 1–6 `#` followed by at least one space, then text
  to end of line. Self-delimiting (a single line). `level` = count of `#`;
  `text` is trimmed.
- **Titled statement**: first line begins with `[label]:` → `title` is the
  label contents, `text` is the remainder of the line plus continuation
  lines.
- **Plain statement**: anything else → `text` is the line(s) until a blank
  line, a heading, or EOF.
- **Multi-line normalization**: trim each line, join with a single space.
- **Comments**: `//` to end of line, `/* … */`, and `<!-- … -->`. Block
  forms may span multiple lines. Comments are recognized and discarded and
  never appear in the AST. A1 supports comments on their own lines, at the
  start/end of content lines, and between blocks; deep mid-word inline
  interleaving inside statement text is deferred to A4 and pinned by tests.
- **Line endings**: `\n` and `\r\n` are both accepted.
- **Indentation**: leading indentation is not yet structural (it becomes
  meaningful for relations in A2); indented content lines are treated like
  any other content line.
- **Empty input**: empty or whitespace-only input yields
  `Document { blocks: [] }`.

## Parser approach (`argdown-parser`, winnow)

- Input is wrapped in `winnow::LocatingSlice<&str>`; spans are captured via
  `with_span()`.
- Module split: `lib.rs` (public `parse` + document assembly), `trivia.rs`
  (whitespace + comments), `heading.rs`, `statement.rs`, `block.rs`
  (dispatch).
- `parse(&str) -> Result<Document, Error>` runs the document parser to full
  consumption (`Parser::parse`) and maps winnow's error to
  `argdown_core::Error`.
- The exact winnow 1.0 API surface (`LocatingSlice`, `with_span`, `parse`,
  error types) is confirmed via context7 at plan-writing time.

## Error model

```rust
pub struct Error { pub message: String, pub offset: usize } // byte position
```

- Replaces the stub `enum Error { Parse(String) }`.
- Implements `Display` (message + offset) and `std::error::Error`.
- Compatibility: `argdown-mcp`'s `eprintln!("failed to parse: {error}")`
  still works (Display), and the existing parser test
  `parse("") == Ok(Document::default())` stays valid (empty input → empty
  document). No `argdown-mcp` change is required.

## Testing

- **Parser** (table-driven, `(input → expected Document)`): empty input,
  single plain statement, titled statement, multi-line wrapped statement,
  each heading level (1–6), multiple blocks, all three comment forms, CRLF
  line endings, and error cases (unterminated `[title`, unterminated block
  comment) asserting `Err` with the correct `offset`.
- **Core**: `Error` `Display` formatting; AST constructors.

## Impact

- Touches `argdown-core` (new AST + `Error`) and `argdown-parser` (real
  grammar).
- Ripple is fully enumerable: `argdown-parser` consumes the new types;
  `argdown-mcp` only `Debug`-prints `Document` and `Display`-prints
  `Error` — both still compile. No `argdown-mcp` change required.
- Small and contained (3-crate repo, dependents enumerated) — blast-radius
  tooling explicitly skipped.

## Success criteria

- `cargo test` passes, including the new table-driven parser tests.
- `parse()` returns a populated `Document` for spine documents and a
  precise `Err { message, offset }` for malformed input.
- `argdown-core` and `argdown-parser` build with no clippy warnings under
  `-D warnings`; formatting is canonical.
