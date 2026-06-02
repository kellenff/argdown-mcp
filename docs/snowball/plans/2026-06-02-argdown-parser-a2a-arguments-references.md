# Argdown Parser A2a (Arguments & References) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add argument definitions/references (`<T>: desc`, `<T>`) and statement references (`[T]`) as block kinds, with a strict "text after a reference is an error" rule, building on the A1 spine.

**Architecture:** Block dispatch becomes head-based: a bracketed head (`[…]`/`<…>`) is parsed first; an immediately-following `:` makes it a definition, otherwise it is a reference and any trailing/continuation text is a hard `cut_err`. Shared line/normalization helpers move into a `text` module so `statement` and `argument` stay DRY. Plain statements keep A1 behavior.

**Tech Stack:** Rust (edition 2024), winnow 1.x (`alt`, `cut_err`, `delimited`, `not`, `opt`, `peek`-free guards via `not`, `take_till`, `take_while`, `till_line_ending`, `line_ending`, `with_span`, `StrContext`).

---

## File Structure

| Path | Responsibility |
|------|----------------|
| `crates/argdown-core/src/ast.rs` | Add `is_reference` to `Statement`; add `Argument`; add `Block::Argument`. (Modify) |
| `crates/argdown-parser/src/text.rs` | Shared helpers: `inline_ws`, `block_head`, `content_line`, `normalize_lines`, `definition_body`, `finish_reference`, `plain_text_line`. (Create) |
| `crates/argdown-parser/src/statement.rs` | `statement` (plain / `[T]:` def / `[T]` ref), `statement_title`. (Rewrite) |
| `crates/argdown-parser/src/argument.rs` | `argument` (`<T>:` def / `<T>` ref), `argument_title`. (Create) |
| `crates/argdown-parser/src/lib.rs` | `mod text;`/`mod argument;`; block dispatch gains the argument arm; tests. (Modify) |
| `crates/argdown-parser/src/trivia.rs` | Unchanged (helpers reused). |
| `crates/argdown-parser/src/heading.rs` | Unchanged. |

---

## Task 1: Add `is_reference` to the AST (keep everything green)

Pure mechanical change: add the field, thread it through the one construction site and the existing test literals. No behavior change. (`Argument` and `Block::Argument` come in Task 3, when they are actually constructed, to avoid an unused-variant warning under `-D warnings`.)

**Files:**
- Modify: `crates/argdown-core/src/ast.rs`
- Modify: `crates/argdown-parser/src/statement.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (test literals)

- [ ] **Step 1: Add the field in `crates/argdown-core/src/ast.rs`**

Replace the `Statement` struct with:

```rust
/// A statement: plain text, a titled definition (`[T]: x`), or a reference (`[T]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub title: Option<String>,
    pub text: String,
    pub is_reference: bool,
    pub span: Span,
}
```

- [ ] **Step 2: Set the field at the construction site in `crates/argdown-parser/src/statement.rs`**

In the `statement` function's returned `Statement { … }`, add `is_reference: false,` (plain statements are never references). The struct literal becomes:

```rust
    Ok(Statement {
        title,
        text: parts.join(" "),
        is_reference: false,
        span: Span { start, end },
    })
```

- [ ] **Step 3: Add `is_reference: false` to every `Statement` literal in the `lib.rs` test module**

In `crates/argdown-parser/src/lib.rs`, the test module has eight `Statement { … }` literals. Add `is_reference: false,` (after the `text:` line) to each of these tests:
`single_plain_statement`, `titled_statement`, `multi_line_statement_is_normalized`, `crlf_within_statement`, `bare_bracket_without_colon_is_plain_text`, `hash_without_space_is_a_statement`, `trailing_line_comment_is_stripped`, `html_comment_is_skipped`.

Example — `single_plain_statement` becomes:

```rust
                blocks: vec![Block::Statement(Statement {
                    title: None,
                    text: "Hello world.".to_string(),
                    is_reference: false,
                    span: Span { start: 0, end: 12 },
                })],
```

- [ ] **Step 4: Verify everything still passes**

Run: `cargo test`
Expected: PASS — `argdown-core` 3, `argdown-parser` 17 (unchanged counts; only the field was added).

- [ ] **Step 5: Commit**

```bash
git add crates
git commit -m "refactor: add is_reference flag to Statement

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Statement references + strict text-after-reference error

Introduce the `text` module (shared helpers), rewrite `statement` to dispatch on a bracketed head, and make `[T]` a reference with the strict error. The A1 `bare_bracket_without_colon_is_plain_text` test reverses to an error.

**Files:**
- Create: `crates/argdown-parser/src/text.rs`
- Rewrite: `crates/argdown-parser/src/statement.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (`mod text;`, tests)

- [ ] **Step 1: Rewrite the reversed test and add reference + error tests in `lib.rs`**

In `crates/argdown-parser/src/lib.rs`, replace the `bare_bracket_without_colon_is_plain_text` test with the reversed version, and append the new tests, all inside the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn text_after_statement_reference_is_an_error() {
        // `[Foo] is text` — what A1 treated as plain text is now an error.
        let err = parse("[Foo] is text").unwrap_err();
        assert_eq!(err.offset, 6);
    }

    #[test]
    fn statement_reference() {
        assert_eq!(
            parse("[S]").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: Some("S".to_string()),
                text: String::new(),
                is_reference: true,
                span: Span { start: 0, end: 3 },
            })]
        );
    }

    #[test]
    fn statement_definition_still_parses() {
        assert_eq!(
            parse("[S]: text").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: Some("S".to_string()),
                text: "text".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 9 },
            })]
        );
    }

    #[test]
    fn two_references_on_adjacent_lines() {
        let blocks = parse("[A]\n[B]").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Statement(s) if s.is_reference));
        assert!(matches!(&blocks[1], Block::Statement(s) if s.is_reference));
    }

    #[test]
    fn text_after_reference_same_line_offset() {
        assert_eq!(parse("[S] words").unwrap_err().offset, 4);
    }

    #[test]
    fn text_after_reference_next_line_offset() {
        assert_eq!(parse("[S]\nwords").unwrap_err().offset, 4);
    }
```

Delete the old `bare_bracket_without_colon_is_plain_text` test (its case is now `text_after_statement_reference_is_an_error`).

- [ ] **Step 2: Add `mod text;` and run to confirm failure**

In `crates/argdown-parser/src/lib.rs`, add `mod text;` next to the other module declarations (the module file doesn't exist yet — that's the next step; this step is just to see the red).

Run: `cargo test -p argdown-parser`
Expected: FAIL to compile (`file not found for module text`) — that's the red signal for this task; proceed to implement.

- [ ] **Step 3: Create `crates/argdown-parser/src/text.rs`**

```rust
//! Shared line helpers: continuation lines, normalization, block boundaries,
//! and the "text after a reference" guard.

use std::ops::Range;

use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, eof, not, opt, repeat};
use winnow::error::StrContext;
use winnow::token::{one_of, take_while};

use crate::Input;
use crate::trivia::{blank_line, comment_start, heading_marker, strip_trailing_line_comment};

/// Consume run of spaces and tabs (no line breaks).
pub(crate) fn inline_ws(input: &mut Input<'_>) -> ModalResult<()> {
    take_while(0.., [' ', '\t']).void().parse_next(input)
}

/// Match a line that begins (after indentation) with `[` or `<` — i.e. the
/// start of a new statement/argument block.
pub(crate) fn block_head(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), one_of(['[', '<']))
        .void()
        .parse_next(input)
}

/// One continuation content line: not EOF, blank, a heading, a comment, or a
/// new block. Returns the raw line (no line ending) and its byte span.
pub(crate) fn content_line<'s>(input: &mut Input<'s>) -> ModalResult<(&'s str, Range<usize>)> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
        not(block_head),
    )
        .parse_next(input)?;
    let (line, span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok((line, span))
}

/// Strip trailing line comments, trim, drop empties, join with a single space.
pub(crate) fn normalize_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> String {
    let mut parts: Vec<&'a str> = Vec::new();
    for line in lines {
        let trimmed = strip_trailing_line_comment(line).trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
}

/// Read a definition body: the remainder of the current line plus continuation
/// content lines. Returns the normalized text and the body's end byte offset.
pub(crate) fn definition_body(input: &mut Input<'_>) -> ModalResult<(String, usize)> {
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);
    let text = normalize_lines(std::iter::once(first).chain(rest.iter().map(|(line, _)| *line)));
    Ok((text, end))
}

/// Succeeds (consuming nothing) when the cursor is at a plain-text line — one
/// that is not EOF, blank, a heading, a comment, or a new block.
fn plain_text_line(input: &mut Input<'_>) -> ModalResult<()> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
        not(block_head),
    )
        .void()
        .parse_next(input)
}

/// Called right after a reference's closing bracket and `inline_ws`. Allows an
/// optional trailing line comment, then requires end-of-line/EOF, then forbids
/// a plain-text continuation line. Emits a hard `cut_err` at the offending text.
pub(crate) fn finish_reference(input: &mut Input<'_>) -> ModalResult<()> {
    opt(("//", till_line_ending).void()).parse_next(input)?;
    cut_err(
        alt((line_ending.void(), eof.void()))
            .context(StrContext::Label("end of reference line (text is not allowed after a reference)")),
    )
    .parse_next(input)?;
    cut_err(not(plain_text_line).context(StrContext::Label("text content after a reference")))
        .parse_next(input)?;
    Ok(())
}
```

- [ ] **Step 4: Rewrite `crates/argdown-parser/src/statement.rs`**

```rust
//! Statements: plain text, titled definitions (`[T]: x`), and references (`[T]`).

use std::ops::Range;

use argdown_core::{Span, Statement};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, delimited, eof, not, opt, repeat};
use winnow::token::take_till;

use crate::Input;
use crate::text::{content_line, definition_body, finish_reference, inline_ws, normalize_lines};
use crate::trivia::blank_line;

/// Parse one statement: a bracketed definition/reference, or plain text.
pub(crate) fn statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    alt((bracketed_statement, plain_statement)).parse_next(input)
}

/// `[Title]: text` (definition) or `[Title]` (reference). Once `[Title]` is
/// consumed the branch is committed, so trailing text is a hard error.
fn bracketed_statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    let (title, span) = statement_title.parse_next(input)?;
    if opt(':').parse_next(input)?.is_some() {
        let (text, end) = definition_body(input)?;
        Ok(Statement {
            title: Some(title),
            text,
            is_reference: false,
            span: Span { start: span.start, end },
        })
    } else {
        inline_ws.parse_next(input)?;
        finish_reference(input)?;
        Ok(Statement {
            title: Some(title),
            text: String::new(),
            is_reference: true,
            span: span.into(),
        })
    }
}

/// `[ title ]` — title trimmed; fails (backtracks) if there is no closing `]`
/// on the same line, so malformed brackets fall through to plain text.
fn statement_title(input: &mut Input<'_>) -> ModalResult<(String, Range<usize>)> {
    delimited('[', take_till(0.., (']', '\r', '\n')), ']')
        .map(|title: &str| title.trim().to_string())
        .with_span()
        .parse_next(input)
}

/// A plain statement: one or more content lines of free text, normalized.
fn plain_statement(input: &mut Input<'_>) -> ModalResult<Statement> {
    (not(eof), not(blank_line)).parse_next(input)?;
    let (first, first_span) = till_line_ending.with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    let rest: Vec<(&str, Range<usize>)> = repeat(0.., content_line).parse_next(input)?;
    let end = rest.last().map_or(first_span.end, |(_, span)| span.end);
    let text = normalize_lines(std::iter::once(first).chain(rest.iter().map(|(line, _)| *line)));
    Ok(Statement {
        title: None,
        text,
        is_reference: false,
        span: Span {
            start: first_span.start,
            end,
        },
    })
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 22 passed` (17 prior − 1 deleted + 6 new = 22).

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-parser
git commit -m "feat: parse statement references with strict text-after-reference error

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Arguments

Add `Argument` to core, create the `argument` parser (definition + reference, reusing `text` helpers), and wire it into block dispatch.

**Files:**
- Modify: `crates/argdown-core/src/ast.rs`
- Create: `crates/argdown-parser/src/argument.rs`
- Modify: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Add argument tests in `lib.rs`**

Add `Argument` to the test imports — change the test `use` line to:

```rust
    use argdown_core::{Argument, Heading, Span, Statement};
```

Append these tests inside the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn argument_definition_single_line() {
        assert_eq!(
            parse("<A>: desc").unwrap().blocks,
            vec![Block::Argument(Argument {
                title: "A".to_string(),
                description: "desc".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 9 },
            })]
        );
    }

    #[test]
    fn argument_definition_multi_line() {
        assert_eq!(
            parse("<A>: one\ntwo").unwrap().blocks,
            vec![Block::Argument(Argument {
                title: "A".to_string(),
                description: "one two".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 12 },
            })]
        );
    }

    #[test]
    fn argument_reference() {
        assert_eq!(
            parse("<A>").unwrap().blocks,
            vec![Block::Argument(Argument {
                title: "A".to_string(),
                description: String::new(),
                is_reference: true,
                span: Span { start: 0, end: 3 },
            })]
        );
    }

    #[test]
    fn text_after_argument_reference_is_an_error() {
        assert_eq!(parse("<A> words").unwrap_err().offset, 4);
        assert_eq!(parse("<A>\nwords").unwrap_err().offset, 4);
    }
```

- [ ] **Step 2: Run to confirm failure**

Run: `cargo test -p argdown-parser`
Expected: FAIL to compile (`cannot find type Argument`, `no variant Argument`) — the red signal.

- [ ] **Step 3: Add `Argument` and `Block::Argument` in `crates/argdown-core/src/ast.rs`**

Add the `Argument` struct (next to `Statement`):

```rust
/// An argument: a titled definition (`<T>: desc`) or a reference (`<T>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    pub title: String,
    pub description: String,
    pub is_reference: bool,
    pub span: Span,
}
```

Add the variant to `Block`:

```rust
pub enum Block {
    Heading(Heading),
    Statement(Statement),
    Argument(Argument),
}
```

Export it from `crates/argdown-core/src/lib.rs`:

```rust
pub use ast::{Argument, Block, Document, Heading, Span, Statement};
```

- [ ] **Step 4: Create `crates/argdown-parser/src/argument.rs`**

```rust
//! Arguments: titled definitions (`<T>: desc`) and references (`<T>`).

use std::ops::Range;

use argdown_core::{Argument, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{delimited, opt};
use winnow::token::take_till;

use crate::Input;
use crate::text::{definition_body, finish_reference, inline_ws};

/// `<Title>: description` (definition) or `<Title>` (reference). Once
/// `<Title>` is consumed the branch is committed; trailing text is an error.
pub(crate) fn argument(input: &mut Input<'_>) -> ModalResult<Argument> {
    let (title, span) = argument_title.parse_next(input)?;
    if opt(':').parse_next(input)?.is_some() {
        let (description, end) = definition_body(input)?;
        Ok(Argument {
            title,
            description,
            is_reference: false,
            span: Span { start: span.start, end },
        })
    } else {
        inline_ws.parse_next(input)?;
        finish_reference(input)?;
        Ok(Argument {
            title,
            description: String::new(),
            is_reference: true,
            span: span.into(),
        })
    }
}

/// `< title >` — title trimmed; fails (backtracks) without a closing `>` on
/// the same line, so an unterminated `<` falls through to plain text.
fn argument_title(input: &mut Input<'_>) -> ModalResult<(String, Range<usize>)> {
    delimited('<', take_till(0.., ('>', '\r', '\n')), '>')
        .map(|title: &str| title.trim().to_string())
        .with_span()
        .parse_next(input)
}
```

- [ ] **Step 5: Wire `argument` into block dispatch in `crates/argdown-parser/src/lib.rs`**

Add the module declaration alongside the others:

```rust
mod argument;
mod heading;
mod statement;
mod text;
mod trivia;
```

Add the import:

```rust
use argument::argument;
use heading::heading;
use statement::statement;
use trivia::skip_trivia;
```

Update `block` to try arguments before statements (a well-formed `<!-- … -->` comment is already consumed as trivia, and an unterminated `<` backtracks into plain text):

```rust
fn block(input: &mut Input<'_>) -> ModalResult<Block> {
    alt((
        heading.map(Block::Heading),
        argument.map(Block::Argument),
        statement.map(Block::Statement),
    ))
    .parse_next(input)
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 26 passed` (22 + 4 new).

- [ ] **Step 7: Commit**

```bash
git add crates
git commit -m "feat: parse argument definitions and references

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Workspace-wide verification

**Files:** none created.

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no output (or harmless reformatting).

- [ ] **Step 2: Verify formatting**

Run: `cargo fmt --check`
Expected: exit 0, no output.

- [ ] **Step 3: Clippy, warnings as errors**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS — `Finished`, no warnings.

- [ ] **Step 4: Build**

Run: `cargo build`
Expected: PASS.

- [ ] **Step 5: Test the workspace**

Run: `cargo test`
Expected: PASS — `argdown-core` 3, `argdown-parser` 26.

- [ ] **Step 6: Run the binary**

Run: `cargo run -p argdown-mcp`
Expected stdout: `parsed argdown document: Document { blocks: [] }`

- [ ] **Step 7: Commit any formatting changes**

```bash
git add -A -- crates
git commit -m "chore: cargo fmt across workspace" || echo "nothing to commit"
```

---

## Success criteria (from the spec)

- `cargo test` passes including the new A2a tests and the reversed bracket test. (Task 4, Step 5)
- `parse()` produces `Argument`/`Statement` definitions and references, and `Err { message, offset }` for text after a reference (same-line and next-line). (Tasks 2–3)
- Clean under `cargo clippy -- -D warnings`; canonical formatting. (Task 4, Steps 2–3)
