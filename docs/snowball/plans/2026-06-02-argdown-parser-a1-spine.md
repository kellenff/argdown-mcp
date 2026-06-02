# Argdown Parser A1 (Spine) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the first real winnow grammar in `argdown-parser` — the document "spine" (headings, plain + titled statements, comments) — producing a populated `argdown_core::Document` with byte spans.

**Architecture:** `argdown-core` holds the Rust-idiomatic syntax tree (`Document` = flat `Vec<Block>`, owned `String` text, byte `Span`s). `argdown-parser` is a winnow grammar over `LocatingSlice<&str>`: a `document` loop parses blocks separated by trivia; `statement` collects consecutive content lines and normalizes them; `heading` parses ATX headings; `trivia` skips whitespace and comments. Strict fail-fast: `parse()` returns `Result<Document, Error>` with a byte offset on the first failure.

**Tech Stack:** Rust (edition 2024), winnow 1.x (`LocatingSlice`, `with_span`, `Parser::parse`, `ModalResult`, combinators `alt`/`opt`/`repeat`/`terminated`/`preceded`/`not`/`eof`, tokens `take_while`/`take_until`/`one_of`, ascii `line_ending`/`till_line_ending`).

---

## File Structure

| Path | Responsibility |
|------|----------------|
| `crates/argdown-core/src/lib.rs` | Module declarations + re-exports. (Modify: replace stub.) |
| `crates/argdown-core/src/ast.rs` | `Span`, `Document`, `Block`, `Heading`, `Statement`, `From<Range> for Span`. (Create.) |
| `crates/argdown-core/src/error.rs` | `Error { message, offset }` + `Display` + `std::error::Error`. (Create.) |
| `crates/argdown-parser/src/lib.rs` | Public `parse`, `document` loop, `block` dispatch, `Input` alias, error mapping, tests. (Modify: replace stub.) |
| `crates/argdown-parser/src/trivia.rs` | Whitespace, blank lines, heading-marker peek, comments, line-comment stripping. (Create.) |
| `crates/argdown-parser/src/statement.rs` | `statement`, `content_line`, `split_title`. (Create.) |
| `crates/argdown-parser/src/heading.rs` | `heading`. (Create.) |

**Naming note:** package `argdown-core` is imported as `argdown_core` (hyphen → underscore).

**TDD note:** Tasks 2–5 (parser logic) follow strict red→green. Task 1 (plain data types) uses a lighter write-type-with-test-then-run flow — there is no behavior to drive out, only data shape.

---

## Task 1: Core AST + Error types

**Files:**
- Modify: `crates/argdown-core/src/lib.rs`
- Create: `crates/argdown-core/src/ast.rs`
- Create: `crates/argdown-core/src/error.rs`

- [ ] **Step 1: Create `crates/argdown-core/src/ast.rs`**

```rust
//! Argdown syntax-tree types.

use std::ops::Range;

/// A byte range into the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
        }
    }
}

/// A parsed Argdown document: a flat sequence of top-level blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// A top-level block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(Heading),
    Statement(Statement),
}

/// An ATX heading (`#`–`######`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub span: Span,
}

/// A statement, optionally titled (`[Title]: text`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub title: Option<String>,
    pub text: String,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_from_range() {
        assert_eq!(Span::from(2..5), Span { start: 2, end: 5 });
    }

    #[test]
    fn document_default_is_empty() {
        assert_eq!(Document::default(), Document { blocks: vec![] });
    }
}
```

- [ ] **Step 2: Create `crates/argdown-core/src/error.rs`**

```rust
//! Error type for Argdown parsing.

/// A parse failure located at a byte offset in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub offset: usize,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_displays_message_and_offset() {
        let err = Error {
            message: "boom".to_string(),
            offset: 7,
        };
        assert_eq!(err.to_string(), "parse error at byte 7: boom");
    }
}
```

- [ ] **Step 3: Replace `crates/argdown-core/src/lib.rs`**

```rust
//! Core domain types for Argdown documents.
//!
//! The syntax-tree types the parser produces and the rest of the program is
//! written against. Grows as the grammar is implemented.

mod ast;
mod error;

pub use ast::{Block, Document, Heading, Span, Statement};
pub use error::Error;
```

- [ ] **Step 4: Run core tests to verify they pass**

Run: `cargo test -p argdown-core`
Expected: PASS — `test result: ok. 3 passed` (span_from_range, document_default_is_empty, error_displays_message_and_offset).

- [ ] **Step 5: Verify the parser crate still compiles against the new core**

Run: `cargo test -p argdown-parser`
Expected: PASS — the existing `parse("") == Ok(Document::default())` test still passes (the stub returns the now-empty `Document`).

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-core
git commit -m "feat: define A1 spine AST and Error types in argdown-core

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Statements (plain + titled, multi-line) and the document loop

**Files:**
- Modify: `crates/argdown-parser/src/lib.rs`
- Create: `crates/argdown-parser/src/trivia.rs`
- Create: `crates/argdown-parser/src/statement.rs`

- [ ] **Step 1: Add the failing tests to `crates/argdown-parser/src/lib.rs`**

Replace the existing `#[cfg(test)] mod tests { … }` block (keep the rest of the stub file as-is for now) with:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use argdown_core::{Span, Statement};

    #[test]
    fn parse_empty_input_yields_empty_document() {
        assert_eq!(parse(""), Ok(Document::default()));
    }

    #[test]
    fn single_plain_statement() {
        assert_eq!(
            parse("Hello world."),
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: None,
                    text: "Hello world.".to_string(),
                    span: Span { start: 0, end: 12 },
                })],
            })
        );
    }

    #[test]
    fn titled_statement() {
        assert_eq!(
            parse("[Key]: Some text"),
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: Some("Key".to_string()),
                    text: "Some text".to_string(),
                    span: Span { start: 0, end: 16 },
                })],
            })
        );
    }

    #[test]
    fn multi_line_statement_is_normalized() {
        assert_eq!(
            parse("Line one\nline two"),
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: None,
                    text: "Line one line two".to_string(),
                    span: Span { start: 0, end: 17 },
                })],
            })
        );
    }

    #[test]
    fn blank_line_separates_statements() {
        let doc = parse("a\n\nb").unwrap();
        assert_eq!(doc.blocks.len(), 2);
    }

    #[test]
    fn crlf_within_statement() {
        assert_eq!(
            parse("a\r\nb").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "a b".to_string(),
                span: Span { start: 0, end: 4 },
            })]
        );
    }

    #[test]
    fn bare_bracket_without_colon_is_plain_text() {
        assert_eq!(
            parse("[Foo] is text").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "[Foo] is text".to_string(),
                span: Span { start: 0, end: 13 },
            })]
        );
    }
}
```

The stub still has `use argdown_core::{Document, Error};` and the `Block` import does not yet exist — add `Block` so the tests compile. At the top of the stub, change the import line to:

```rust
use argdown_core::{Block, Document, Error};
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser`
Expected: FAIL — `single_plain_statement`, `titled_statement`, `multi_line_statement_is_normalized`, `blank_line_separates_statements`, `crlf_within_statement`, and `bare_bracket_without_colon_is_plain_text` fail their assertions (the stub returns an empty `Document`). `parse_empty_input_yields_empty_document` still passes.

- [ ] **Step 3: Create `crates/argdown-parser/src/trivia.rs`**

```rust
//! Whitespace, blank lines, heading-marker detection, and comment helpers.

use winnow::Parser;
use winnow::ModalResult;
use winnow::ascii::line_ending;
use winnow::token::{one_of, take_while};

use crate::Input;

/// Skip inter-block trivia: runs of whitespace and line breaks.
pub(crate) fn skip_trivia(input: &mut Input<'_>) -> ModalResult<()> {
    take_while(0.., [' ', '\t', '\r', '\n'])
        .void()
        .parse_next(input)
}

/// Match a blank line (only whitespace, then a line ending).
pub(crate) fn blank_line(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), line_ending)
        .void()
        .parse_next(input)
}

/// Match the start of an ATX heading: 1–6 `#` followed by a space or tab.
pub(crate) fn heading_marker(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(1..=6, '#'), one_of([' ', '\t']))
        .void()
        .parse_next(input)
}

/// Remove a trailing `// …` line comment from raw line text.
pub(crate) fn strip_trailing_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(index) => &line[..index],
        None => line,
    }
}
```

- [ ] **Step 4: Create `crates/argdown-parser/src/statement.rs`**

```rust
//! Plain and titled statements, possibly spanning multiple wrapped lines.

use std::ops::Range;

use argdown_core::{Span, Statement};
use winnow::Parser;
use winnow::ModalResult;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{eof, not, opt, repeat};

use crate::Input;
use crate::trivia::{blank_line, heading_marker, strip_trailing_line_comment};

/// Parse one statement: one or more consecutive content lines, normalized.
pub(crate) fn statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    let lines: Vec<(&str, Range<usize>)> = repeat(1.., content_line).parse_next(input)?;
    let start = lines.first().expect("repeat(1..) yields >= 1 line").1.start;
    let end = lines.last().expect("repeat(1..) yields >= 1 line").1.end;

    let cleaned: Vec<&str> = lines
        .iter()
        .map(|(line, _)| strip_trailing_line_comment(line))
        .collect();

    let (title, first_rest) = split_title(cleaned[0]);

    let mut parts: Vec<&str> = Vec::new();
    let first = first_rest.trim();
    if !first.is_empty() {
        parts.push(first);
    }
    for line in &cleaned[1..] {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    Ok(Statement {
        title,
        text: parts.join(" "),
        span: Span { start, end },
    })
}

/// One content line: not EOF, not blank, not a heading. Returns the raw line
/// (without its line ending) and the byte span of that text.
fn content_line<'s>(input: &mut Input<'s>) -> ModalResult<(&'s str, Range<usize>)> {
    (not(eof), not(blank_line), not(heading_marker)).parse_next(input)?;
    let (line, span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok((line, span))
}

/// Split a leading `[Title]:` off a line. A bare `[…]` without `]:` is plain
/// text (statement references arrive in increment A2).
fn split_title(line: &str) -> (Option<String>, &str) {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix('[')
        && let Some(close) = rest.find("]:")
    {
        let title = rest[..close].trim().to_string();
        return (Some(title), &rest[close + 2..]);
    }
    (None, line)
}
```

- [ ] **Step 5: Replace the non-test portion of `crates/argdown-parser/src/lib.rs`**

Replace everything above the `#[cfg(test)]` test module (added in Step 1) with:

```rust
//! Winnow-based parser for the Argdown format (increment A1: spine).
//!
//! Parses headings, plain and titled statements, and comments into an
//! [`argdown_core::Document`]. See the A1 spine design spec.

mod statement;
mod trivia;

use argdown_core::{Block, Document, Error};
use winnow::Parser;
use winnow::ModalResult;
use winnow::combinator::{repeat, terminated};
use winnow::stream::LocatingSlice;

use statement::statement;
use trivia::skip_trivia;

/// The winnow input stream: `&str` augmented with byte-offset locations.
pub(crate) type Input<'s> = LocatingSlice<&'s str>;

/// Parse Argdown source text into a [`Document`].
pub fn parse(source: &str) -> Result<Document, Error> {
    document
        .parse(Input::new(source))
        .map_err(|e| Error {
            message: e.to_string(),
            offset: e.offset(),
        })
}

fn document(input: &mut Input<'_>) -> ModalResult<Document> {
    skip_trivia(input)?;
    let blocks: Vec<Block> = repeat(0.., terminated(block, skip_trivia)).parse_next(input)?;
    Ok(Document { blocks })
}

fn block(input: &mut Input<'_>) -> ModalResult<Block> {
    statement.map(Block::Statement).parse_next(input)
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 7 passed`.

- [ ] **Step 7: Commit**

```bash
git add crates/argdown-parser
git commit -m "feat: parse plain and titled statements with spans

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Headings

**Files:**
- Create: `crates/argdown-parser/src/heading.rs`
- Modify: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Add the failing tests to `crates/argdown-parser/src/lib.rs`**

Inside the `#[cfg(test)] mod tests` block, add `Heading` to the test imports and append the new tests. Change the test import line to:

```rust
    use argdown_core::{Heading, Span, Statement};
```

Append these tests inside the `tests` module:

```rust
    #[test]
    fn heading_level_one() {
        assert_eq!(
            parse("# Title").unwrap().blocks,
            vec![Block::Heading(Heading {
                level: 1,
                text: "Title".to_string(),
                span: Span { start: 0, end: 7 },
            })]
        );
    }

    #[test]
    fn heading_levels_two_through_six() {
        for level in 2u8..=6 {
            let hashes = "#".repeat(level as usize);
            let source = format!("{hashes} Deep");
            let blocks = parse(&source).unwrap().blocks;
            assert_eq!(
                blocks,
                vec![Block::Heading(Heading {
                    level,
                    text: "Deep".to_string(),
                    span: Span {
                        start: 0,
                        end: source.len(),
                    },
                })]
            );
        }
    }

    #[test]
    fn heading_then_statement_without_blank_line() {
        let blocks = parse("# Title\nbody").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0], Block::Heading(_)));
        assert!(matches!(blocks[1], Block::Statement(_)));
    }

    #[test]
    fn hash_without_space_is_a_statement() {
        let blocks = parse("#nospace").unwrap().blocks;
        assert_eq!(
            blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "#nospace".to_string(),
                span: Span { start: 0, end: 8 },
            })]
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser`
Expected: FAIL — `heading_level_one`, `heading_levels_two_through_six`, and `heading_then_statement_without_blank_line` fail (heading lines currently produce a parse error because no `heading` block exists, so `parse` returns `Err`). `hash_without_space_is_a_statement` passes already.

- [ ] **Step 3: Create `crates/argdown-parser/src/heading.rs`**

```rust
//! ATX headings (`#`–`######`).

use argdown_core::Heading;
use winnow::Parser;
use winnow::ModalResult;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{opt, preceded};
use winnow::token::take_while;

use crate::Input;
use crate::trivia::strip_trailing_line_comment;

/// Parse one ATX heading: 1–6 `#`, at least one space/tab, then text to EOL.
pub(crate) fn heading(input: &mut Input<'_>) -> ModalResult<Heading> {
    let ((level, raw), span) = (
        take_while(1..=6, '#').map(|hashes: &str| hashes.len() as u8),
        preceded(take_while(1.., [' ', '\t']), till_line_ending),
    )
        .with_span()
        .parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok(Heading {
        level,
        text: strip_trailing_line_comment(raw).trim().to_string(),
        span: span.into(),
    })
}
```

- [ ] **Step 4: Wire `heading` into the block dispatcher in `crates/argdown-parser/src/lib.rs`**

Add the module declaration alongside the others:

```rust
mod heading;
mod statement;
mod trivia;
```

Add the import alongside the others:

```rust
use heading::heading;
use statement::statement;
use trivia::skip_trivia;
```

Add `alt` to the combinator import:

```rust
use winnow::combinator::{alt, repeat, terminated};
```

Replace the `block` function with:

```rust
fn block(input: &mut Input<'_>) -> ModalResult<Block> {
    alt((
        heading.map(Block::Heading),
        statement.map(Block::Statement),
    ))
    .parse_next(input)
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 11 passed`.

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-parser
git commit -m "feat: parse ATX headings

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Comments

A1 supports: line comments (`// …`), block comments (`/* … */`), and HTML
comments (`<!-- … -->`) as standalone trivia between/around blocks (block and
HTML forms may span lines), plus a trailing `// …` line comment on a content
or heading line (already stripped via `strip_trailing_line_comment`). Trailing
block/HTML comments embedded mid-line, and `//` inside intended text (e.g.
URLs), are deferred to increment A4.

**Files:**
- Modify: `crates/argdown-parser/src/trivia.rs`
- Modify: `crates/argdown-parser/src/statement.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (tests only)

- [ ] **Step 1: Add the failing tests to `crates/argdown-parser/src/lib.rs`**

Append these tests inside the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn line_comment_between_statements_is_skipped() {
        let blocks = parse("a\n// note\nb").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn trailing_line_comment_is_stripped() {
        assert_eq!(
            parse("foo // bar").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "foo".to_string(),
                span: Span { start: 0, end: 10 },
            })]
        );
    }

    #[test]
    fn block_comment_spanning_lines_is_skipped() {
        let blocks = parse("a\n/* x\ny */\nb").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn html_comment_is_skipped() {
        let blocks = parse("<!-- c -->\nb").unwrap().blocks;
        assert_eq!(
            blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "b".to_string(),
                span: Span { start: 11, end: 12 },
            })]
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser`
Expected: FAIL — `line_comment_between_statements_is_skipped`, `block_comment_spanning_lines_is_skipped`, and `html_comment_is_skipped` fail. (Without the `comment_start` guard, a comment line that is not separated by a blank line gets absorbed into a single multi-line statement, so e.g. `line_comment_between_statements_is_skipped` sees 1 block instead of 2; `html_comment_is_skipped` sees the comment and `b` merged into one statement.) `trailing_line_comment_is_stripped` passes already (stripping is in place from Task 2).

- [ ] **Step 3: Replace `crates/argdown-parser/src/trivia.rs`**

```rust
//! Whitespace, blank lines, heading-marker detection, and comments.

use winnow::Parser;
use winnow::ModalResult;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, repeat};
use winnow::token::{one_of, take_until, take_while};

use crate::Input;

/// Skip inter-block trivia: runs of whitespace, line breaks, and comments.
pub(crate) fn skip_trivia(input: &mut Input<'_>) -> ModalResult<()> {
    let _: () = repeat(
        0..,
        alt((
            take_while(1.., [' ', '\t', '\r', '\n']).void(),
            comment,
        )),
    )
    .parse_next(input)?;
    Ok(())
}

/// Consume one comment: line (`// …`), block (`/* … */`), or HTML
/// (`<!-- … -->`). Block and HTML forms may span multiple lines. Fails (with
/// the cursor at the opener) if a block/HTML comment is never closed.
pub(crate) fn comment(input: &mut Input<'_>) -> ModalResult<()> {
    alt((
        ("//", till_line_ending).void(),
        ("/*", take_until(0.., "*/"), "*/").void(),
        ("<!--", take_until(0.., "-->"), "-->").void(),
    ))
    .parse_next(input)
}

/// Match a blank line (only whitespace, then a line ending).
pub(crate) fn blank_line(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), line_ending)
        .void()
        .parse_next(input)
}

/// Match the start of an ATX heading: 1–6 `#` followed by a space or tab.
pub(crate) fn heading_marker(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(1..=6, '#'), one_of([' ', '\t']))
        .void()
        .parse_next(input)
}

/// Match the start of a comment at the beginning of a line (after optional
/// indentation).
pub(crate) fn comment_start(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), alt(("//", "/*", "<!--")))
        .void()
        .parse_next(input)
}

/// Remove a trailing `// …` line comment from raw line text.
pub(crate) fn strip_trailing_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(index) => &line[..index],
        None => line,
    }
}
```

- [ ] **Step 4: Add the comment guard to `content_line` in `crates/argdown-parser/src/statement.rs`**

Change the trivia import to include `comment_start`:

```rust
use crate::trivia::{blank_line, comment_start, heading_marker, strip_trailing_line_comment};
```

Replace the guard line in `content_line` (the first line of its body) so it reads:

```rust
    (not(eof), not(blank_line), not(heading_marker), not(comment_start)).parse_next(input)?;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 15 passed`.

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-parser
git commit -m "feat: recognize and skip comments

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Strict error reporting

With the lenient spine grammar, most malformed input degrades to plain
statements by design (a bare `[label]` is text, a lone `#` is text). The
genuine hard failure is an **unterminated block or HTML comment**: `take_until`
finds no closer, the document parser stops, and `Parser::parse` reports the
leftover input at the opener's byte offset.

**Files:**
- Modify: `crates/argdown-parser/src/lib.rs` (tests only)

- [ ] **Step 1: Add the failing tests to `crates/argdown-parser/src/lib.rs`**

Append these tests inside the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn unterminated_block_comment_errors_at_opener() {
        let err = parse("/* oops").unwrap_err();
        assert_eq!(err.offset, 0);
    }

    #[test]
    fn error_offset_points_past_earlier_blocks() {
        let err = parse("foo\n/* x").unwrap_err();
        assert_eq!(err.offset, 4);
    }
```

- [ ] **Step 2: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 17 passed`. (These assertions already hold given Task 4's `take_until` behavior and the offset mapping in `parse`; this task pins that contract with tests.)

If either test fails, STOP — the error-offset contract is wrong and must be fixed before continuing, not worked around.

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-parser
git commit -m "test: pin strict error offsets for unterminated comments

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Workspace-wide verification

**Files:** none created; formats, lints, builds, tests, and runs the workspace.

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no output (or harmless reformatting).

- [ ] **Step 2: Verify formatting is canonical**

Run: `cargo fmt --check`
Expected: no output, exit code 0.

- [ ] **Step 3: Lint with clippy, warnings as errors**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS — `Finished` with no warnings.

- [ ] **Step 4: Build the whole workspace**

Run: `cargo build`
Expected: PASS — all three crates compile.

- [ ] **Step 5: Test the whole workspace**

Run: `cargo test`
Expected: PASS — `argdown-core` (3 tests) and `argdown-parser` (17 tests) pass; `argdown-mcp` has none.

- [ ] **Step 6: Run the binary to confirm it still works end-to-end**

Run: `cargo run -p argdown-mcp`
Expected stdout: `parsed argdown document: Document { blocks: [] }` (the binary parses the empty string).

- [ ] **Step 7: Commit any formatting changes**

```bash
git add -A -- crates
git commit -m "chore: cargo fmt across workspace" || echo "nothing to commit"
```

Expected: a formatting commit, or `nothing to commit`.

---

## Success criteria (from the spec)

- `cargo test` passes including the table-driven parser tests. (Task 6, Step 5)
- `parse()` returns a populated `Document` for spine documents and a precise
  `Err { message, offset }` for unterminated comments. (Tasks 2–5)
- `argdown-core` and `argdown-parser` build clean under
  `cargo clippy -- -D warnings`; formatting is canonical. (Task 6, Steps 2–3)
