# Argdown Parser A5b (Document Frontmatter) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recognize a single document-level `===…===` YAML frontmatter block at the start of an Argdown document and capture it as `Document.frontmatter: Option<Metadata>` (raw inner content + absolute source span), deferring the YAML parse to Layer B.

**Architecture:** Add a `frontmatter` field to the `Document` AST node (reusing the A5a `Metadata { raw, span }` type). A new `frontmatter.rs` parser module provides a backtrackable `fence_marker` (the single definition of a `===` fence line), a `fence_line` lookahead, and the `frontmatter` recognizer. `document()` tries `opt(frontmatter)` once after leading trivia. Two hooks enforce "fences only at document start": `at_content_line` stops continuation readers at a fence line, and `block()` rejects a fence line at any later block boundary.

**Tech Stack:** Rust, [winnow](https://docs.rs/winnow) parser-combinators (`LocatingSlice` input for absolute byte spans), `cargo test` / `cargo clippy` / `cargo fmt`.

---

## File Structure

- **Modify** `crates/argdown-core/src/ast.rs` — add `frontmatter: Option<Metadata>` to `Document`; update the one test literal.
- **Create** `crates/argdown-parser/src/frontmatter.rs` — `fence_marker`, `fence_line` (exported `pub(crate)`), `body_line`, `frontmatter`, plus module unit tests.
- **Modify** `crates/argdown-parser/src/lib.rs` — register `mod frontmatter;`; wire `opt(frontmatter)` into `document()`; add the `misplaced_fence` cut-branch to `block()`; update three test literals; add integration tests.
- **Modify** `crates/argdown-parser/src/text.rs` — add `not(fence_line)` to `at_content_line`.

Reference patterns to mirror: `crates/argdown-parser/src/metadata.rs` (a self-contained recognizer with module unit tests), `crates/argdown-parser/src/heading.rs` (cut-error style), `crates/argdown-parser/src/text.rs` (`inline_ws`, `at_content_line`).

---

### Task 1: Add `Document.frontmatter` field (additive churn)

**Files:**
- Modify: `crates/argdown-core/src/ast.rs:21-25` (struct), `:179` (test literal)
- Modify: `crates/argdown-parser/src/lib.rs:44` (constructor), `:231`, `:248`, `:265` (test literals)

This task only adds the field and makes everything compile with `frontmatter: None`. No behavior change.

- [ ] **Step 1: Add the field to `Document`**

In `crates/argdown-core/src/ast.rs`, replace the `Document` struct (lines 21-25):

```rust
/// A parsed Argdown document: optional `===…===` frontmatter plus a flat
/// sequence of top-level blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
    /// The leading `===…===` YAML frontmatter block, if present. Raw content +
    /// span; the YAML is not parsed here (a Layer-B utility does that).
    pub frontmatter: Option<Metadata>,
}
```

- [ ] **Step 2: Fix the `ast.rs` test literal**

In `crates/argdown-core/src/ast.rs`, the `document_default_is_empty` test (line ~179) compares against a `Document` literal. Update it:

```rust
    #[test]
    fn document_default_is_empty() {
        assert_eq!(
            Document::default(),
            Document {
                blocks: vec![],
                frontmatter: None,
            }
        );
    }
```

- [ ] **Step 3: Fix the `document()` constructor (temporary `None`)**

In `crates/argdown-parser/src/lib.rs`, the `document` function ends with `Ok(Document { blocks })` (line ~44). Change it to:

```rust
    Ok(Document {
        blocks,
        frontmatter: None,
    })
```

(Task 3 replaces the `None` with the real recognizer result.)

- [ ] **Step 4: Fix the three `lib.rs` test literals**

In `crates/argdown-parser/src/lib.rs`, three tests assert against a full `Document { blocks: … }` literal: `single_plain_statement` (~line 231), `titled_statement` (~248), and `multi_line_statement_is_normalized` (~265). Add `frontmatter: None,` after the `blocks: vec![…]` field in each. Example for `single_plain_statement`:

```rust
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: None,
                    text: "Hello world.".to_string(),
                    is_reference: false,
                    span: Span { start: 0, end: 12 },
                    inlines: vec![],
                    metadata: None,
                })],
                frontmatter: None,
            })
```

Apply the same `frontmatter: None,` addition to `titled_statement` and `multi_line_statement_is_normalized`.

- [ ] **Step 5: Run the full suite — everything still passes**

Run: `cargo test -p argdown-parser -p argdown-core`
Expected: PASS (all existing tests green; the field addition is purely additive).

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-core/src/ast.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: add Document.frontmatter field (A5b scaffold)"
```

---

### Task 2: The `frontmatter.rs` recognizer module

**Files:**
- Create: `crates/argdown-parser/src/frontmatter.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (add `mod frontmatter;`)

We write the module with its own unit tests (testing the parsers directly on an `Input`). It is not yet wired into `document()`, so `cargo test` will report dead-code *warnings* for `frontmatter`/`fence_line` — that is expected and harmless until Tasks 3–4. (Clippy `-D warnings` is deferred to Task 5, by which point everything is wired.)

- [ ] **Step 1: Write the module with a failing unit test first**

Create `crates/argdown-parser/src/frontmatter.rs` with the test module only, plus stub signatures so it compiles:

```rust
//! Document frontmatter recognition: a leading `===…===` block whose inner
//! content is captured raw (YAML not parsed here). `fence_marker` is the single
//! definition of a fence line, reused for the open fence, the close fence, and
//! the `at_content_line` / `block()` guards that keep fences at document start.

use std::ops::Range;

use argdown_core::{Metadata, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{alt, cut_err, eof, not, opt, peek, repeat};
use winnow::token::take_while;

use crate::Input;
use crate::text::inline_ws;
use crate::trivia::blank_line;

#[cfg(test)]
mod tests {
    use super::*;

    fn run_frontmatter(src: &str) -> Result<Metadata, ()> {
        let mut input = Input::new(src);
        frontmatter(&mut input).map_err(|_| ())
    }

    #[test]
    fn captures_basic_block() {
        let m = run_frontmatter("===\ntitle: X\nauthor: Y\n===\n").unwrap();
        assert_eq!(m.raw, "title: X\nauthor: Y\n");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails to compile**

Run: `cargo test -p argdown-parser frontmatter`
Expected: FAIL — `cannot find function 'frontmatter' in this scope`.

- [ ] **Step 3: Implement `fence_marker`, `fence_line`, `body_line`, `frontmatter`**

Add these above the `#[cfg(test)]` block in `crates/argdown-parser/src/frontmatter.rs`:

```rust
/// Match one fence line and return the byte range of its `=` run. A fence line
/// is: optional leading whitespace, three or more `=`, optional trailing
/// whitespace, then a line ending or EOF. Backtrackable: fewer than three `=`,
/// or trailing non-whitespace, fails so the line is treated as ordinary content.
fn fence_marker(input: &mut Input<'_>) -> ModalResult<Range<usize>> {
    inline_ws(input)?;
    let (_, span) = take_while(3.., '=').with_span().parse_next(input)?;
    inline_ws(input)?;
    alt((line_ending.void(), eof.void())).parse_next(input)?;
    Ok(span)
}

/// Lookahead form of `fence_marker`: succeeds (consuming the line) when the
/// current line is a fence line. Used via `not(fence_line)` / `peek(fence_line)`.
pub(crate) fn fence_line(input: &mut Input<'_>) -> ModalResult<()> {
    fence_marker(input)?;
    Ok(())
}

/// One raw frontmatter body line: any line that is neither a fence line nor EOF.
/// Backtracks (consuming nothing) at the closing fence or at EOF so the body
/// `repeat` stops there.
fn body_line(input: &mut Input<'_>) -> ModalResult<()> {
    not(fence_line).parse_next(input)?;
    not(eof).parse_next(input)?;
    till_line_ending.void().parse_next(input)?;
    opt(line_ending).void().parse_next(input)?;
    Ok(())
}

/// Recognize a leading `===…===` frontmatter block. Returns `Metadata` where
/// `raw` is the verbatim body between the fences (line endings preserved, both
/// fence lines excluded) and `span` runs from the first `=` of the open fence to
/// just past the last `=` of the close fence. The open fence is backtrackable
/// (so `opt(frontmatter)` yields `None` on a non-frontmatter document); after it
/// matches, failures are hard `Cut` errors: no closing fence before EOF
/// (unterminated), or content on the line immediately after the close (a missing
/// paragraph break).
pub(crate) fn frontmatter(input: &mut Input<'_>) -> ModalResult<Metadata> {
    let open = fence_marker(input)?;
    let (_, raw): ((), &str) = repeat(0.., body_line).with_taken().parse_next(input)?;
    let close = cut_err(fence_marker).parse_next(input)?;
    cut_err(peek(alt((
        eof.void(),
        blank_line.void(),
        (inline_ws, eof).void(),
    ))))
    .parse_next(input)?;
    Ok(Metadata {
        raw: raw.to_string(),
        span: Span {
            start: open.start,
            end: close.end,
        },
    })
}
```

- [ ] **Step 4: Register the module**

In `crates/argdown-parser/src/lib.rs`, add `mod frontmatter;` to the module declarations (alphabetical, after `mod argument;`/`mod heading;` — place it before `mod inline;`):

```rust
mod argument;
mod frontmatter;
mod heading;
mod inline;
mod metadata;
mod pcs;
mod relation;
mod statement;
mod text;
mod trivia;
```

- [ ] **Step 5: Run the unit test — it passes**

Run: `cargo test -p argdown-parser frontmatter`
Expected: PASS (`captures_basic_block`). Dead-code warnings for `frontmatter`/`fence_line` are expected here.

- [ ] **Step 6: Add the remaining module unit tests**

Append to the `tests` module in `frontmatter.rs`:

```rust
    #[test]
    fn span_covers_the_whole_fenced_block() {
        let src = "===\ntitle: X\nauthor: Y\n===\n";
        let mut input = Input::new(src);
        let m = frontmatter(&mut input).unwrap();
        assert_eq!(&src[m.span.start..m.span.end], "===\ntitle: X\nauthor: Y\n===");
    }

    #[test]
    fn empty_body_is_captured() {
        let m = run_frontmatter("===\n===\n").unwrap();
        assert_eq!(m.raw, "");
    }

    #[test]
    fn crlf_body_preserves_line_endings() {
        let src = "===\r\ntitle: X\r\n===\r\n";
        let mut input = Input::new(src);
        let m = frontmatter(&mut input).unwrap();
        assert_eq!(m.raw, "title: X\r\n");
        assert_eq!(&src[m.span.start..m.span.end], "===\r\ntitle: X\r\n===");
    }

    #[test]
    fn four_or_more_equals_is_a_fence() {
        assert!(run_frontmatter("====\na: b\n====\n").is_ok());
    }

    #[test]
    fn indented_fence_is_a_fence() {
        let m = run_frontmatter("  ===\na: b\n  ===\n").unwrap();
        assert_eq!(m.raw, "a: b\n");
    }

    #[test]
    fn non_yaml_body_is_still_captured() {
        // The recognizer never parses YAML; any body text is captured verbatim.
        let m = run_frontmatter("===\nthis is: not: valid: yaml\n===\n").unwrap();
        assert_eq!(m.raw, "this is: not: valid: yaml\n");
    }

    #[test]
    fn two_equals_is_not_a_fence() {
        // `==` opens no frontmatter: fence_marker backtracks, frontmatter errors.
        assert!(run_frontmatter("==\na: b\n==\n").is_err());
    }

    #[test]
    fn unterminated_block_is_an_error() {
        assert!(run_frontmatter("===\ntitle: X\n").is_err());
    }

    #[test]
    fn eof_immediately_after_close_is_ok() {
        // No trailing newline at all after the closing fence.
        assert!(run_frontmatter("===\na: b\n===").is_ok());
    }
```

- [ ] **Step 7: Run all module unit tests**

Run: `cargo test -p argdown-parser frontmatter`
Expected: PASS (all nine frontmatter unit tests).

- [ ] **Step 8: Commit**

```bash
git add crates/argdown-parser/src/frontmatter.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: frontmatter recognizer (fence_marker, fence_line, frontmatter)"
```

---

### Task 3: Wire `frontmatter` into `document()`

**Files:**
- Modify: `crates/argdown-parser/src/lib.rs` (`document()`, imports, integration tests)

- [ ] **Step 1: Write failing integration tests**

In `crates/argdown-parser/src/lib.rs`, add to the `tests` module (alongside the existing metadata tests):

```rust
    #[test]
    fn frontmatter_at_document_start() {
        let doc = parse("===\ntitle: X\nauthor: Y\n===\n\n[S]: claim").unwrap();
        let fm = doc.frontmatter.as_ref().expect("frontmatter");
        assert_eq!(fm.raw, "title: X\nauthor: Y\n");
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(&doc.blocks[0], Block::Statement(s) if s.title.as_deref() == Some("S")));
    }

    #[test]
    fn frontmatter_span_is_absolute_after_leading_blank_line() {
        let src = "\n\n===\ntitle: X\n===\n\n[S]: claim";
        let doc = parse(src).unwrap();
        let fm = doc.frontmatter.as_ref().expect("frontmatter");
        assert_eq!(&src[fm.span.start..fm.span.end], "===\ntitle: X\n===");
    }

    #[test]
    fn leading_comment_before_frontmatter_is_fine() {
        let doc = parse("// hello\n===\ntitle: X\n===\n\n[S]: claim").unwrap();
        assert!(doc.frontmatter.is_some());
        assert_eq!(doc.blocks.len(), 1);
    }

    #[test]
    fn document_without_frontmatter_has_none() {
        let doc = parse("[S]: claim").unwrap();
        assert!(doc.frontmatter.is_none());
    }

    #[test]
    fn frontmatter_only_document_has_no_blocks() {
        let doc = parse("===\ntitle: X\n===\n").unwrap();
        assert!(doc.frontmatter.is_some());
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn content_immediately_after_close_fence_is_an_error() {
        // D3: a blank line (or EOF) must follow the closing fence.
        assert!(parse("===\ntitle: X\n===\n[S]: claim").is_err());
    }

    #[test]
    fn unterminated_frontmatter_is_an_error() {
        // D4: opening fence with no closing fence before EOF.
        assert!(parse("===\ntitle: X\n[S]: claim").is_err());
    }
```

- [ ] **Step 2: Run them to verify failure**

Run: `cargo test -p argdown-parser frontmatter_at_document_start`
Expected: FAIL — `frontmatter` is still hard-coded to `None` in `document()`, so `expect("frontmatter")` panics.

- [ ] **Step 3: Add `opt` and `frontmatter` to the imports**

In `crates/argdown-parser/src/lib.rs`, update the `winnow::combinator` import and add the module import:

```rust
use winnow::combinator::{alt, opt, repeat, terminated};
```

And in the per-module `use` block (after `use argument::argument;`), add:

```rust
use frontmatter::frontmatter;
```

- [ ] **Step 4: Wire `opt(frontmatter)` into `document()`**

Replace the body of `document` in `crates/argdown-parser/src/lib.rs`:

```rust
fn document(input: &mut Input<'_>) -> ModalResult<Document> {
    skip_trivia(input)?;
    let frontmatter = opt(frontmatter).parse_next(input)?;
    skip_trivia(input)?;
    let blocks: Vec<Block> = repeat(0.., terminated(block, skip_trivia)).parse_next(input)?;
    Ok(Document { blocks, frontmatter })
}
```

- [ ] **Step 5: Run the integration tests**

Run: `cargo test -p argdown-parser`
Expected: PASS (the seven new tests plus all prior tests). `fence_line` is still unused → a dead-code warning remains until Task 4.

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-parser/src/lib.rs
git commit -m "feat: recognize document frontmatter in document()"
```

---

### Task 4: Enforce "fences only at document start" (D2)

**Files:**
- Modify: `crates/argdown-parser/src/text.rs` (`at_content_line`, imports)
- Modify: `crates/argdown-parser/src/lib.rs` (`block()`, `misplaced_fence`, imports, tests)

- [ ] **Step 1: Write failing tests for misplaced fences**

In `crates/argdown-parser/src/lib.rs` `tests` module, add:

```rust
    #[test]
    fn fence_after_content_is_an_error() {
        // D2: frontmatter is only valid at document start.
        assert!(parse("[S]: x\n\n===\ntitle: X\n===\n").is_err());
    }

    #[test]
    fn bare_fence_after_content_is_an_error() {
        assert!(parse("a claim\n===").is_err());
    }

    #[test]
    fn fence_does_not_get_absorbed_as_statement_continuation() {
        // The continuation reader stops at the fence line; the fence then
        // surfaces as the misplaced-fence error rather than joining the prose.
        assert!(parse("first line\nsecond line\n===").is_err());
    }
```

- [ ] **Step 2: Run them to verify failure**

Run: `cargo test -p argdown-parser fence_after_content_is_an_error`
Expected: FAIL — without the guard, `===` after content is swallowed as statement text (`parse` returns `Ok`), so `assert!(… .is_err())` fails.

- [ ] **Step 3: Stop continuation readers at a fence line**

In `crates/argdown-parser/src/text.rs`, add the import (with the other `crate::` imports near the top):

```rust
use crate::frontmatter::fence_line;
```

Then add `not(fence_line)` to `at_content_line` (the tuple of negative lookaheads):

```rust
fn at_content_line(input: &mut Input<'_>) -> ModalResult<()> {
    (
        not(eof),
        not(blank_line),
        not(heading_marker),
        not(comment_start),
        not(block_head),
        not(relation_marker),
        not(pcs_marker),
        not(fence_line),
    )
        .void()
        .parse_next(input)
}
```

- [ ] **Step 4: Add the `misplaced_fence` cut-branch to `block()`**

In `crates/argdown-parser/src/lib.rs`, extend the imports:

```rust
use winnow::combinator::{alt, opt, peek, repeat, terminated};
use winnow::error::{ContextError, ErrMode};
```

and add `fence_line` to the frontmatter import:

```rust
use frontmatter::{fence_line, frontmatter};
```

Add `misplaced_fence` as the first alternative in `block()` and define it:

```rust
fn block(input: &mut Input<'_>) -> ModalResult<Block> {
    alt((
        misplaced_fence,
        heading.map(Block::Heading),
        relation.map(Block::Relation),
        pcs.map(Block::Pcs),
        argument.map(Block::Argument),
        statement.map(Block::Statement),
    ))
    .parse_next(input)
}

/// A fence line reaching a block boundary is a misplaced frontmatter fence
/// (frontmatter is only valid at document start) — a hard error. Backtracks when
/// the line is not a fence so the normal block alternatives run.
fn misplaced_fence(input: &mut Input<'_>) -> ModalResult<Block> {
    peek(fence_line).parse_next(input)?;
    Err(ErrMode::Cut(ContextError::new()))
}
```

- [ ] **Step 5: Run the new tests and the full suite**

Run: `cargo test -p argdown-parser`
Expected: PASS (the three D2 tests plus all prior tests).

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-parser/src/text.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: reject frontmatter fences after document start (A5b D2)"
```

---

### Task 5: Final verification (lint, format, full workspace)

**Files:** none (verification + any formatting fixups)

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no diff, or only canonical-formatting fixups. If files change, they are formatting-only.

- [ ] **Step 2: Clippy with warnings-as-errors (catches any remaining dead code)**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS, no warnings. (All of `fence_marker`, `fence_line`, `body_line`, `frontmatter` are now used, so no dead-code warnings remain.)

- [ ] **Step 3: Full workspace test run**

Run: `cargo test --workspace`
Expected: PASS — all `argdown-core`, `argdown-parser`, and `argdown-mcp` tests green.

- [ ] **Step 4: Sanity-check `argdown-mcp` still builds and prints frontmatter**

Run: `cargo build --workspace`
Expected: PASS. (`argdown-mcp` only `Debug`-prints `Document`, so the new field appears automatically with no code change.)

- [ ] **Step 5: Commit any formatting fixups**

```bash
git add -A
git commit -m "chore: fmt/clippy clean for A5b frontmatter" || echo "nothing to commit"
```

---

## Self-Review

**Spec coverage** — every spec section maps to a task:
- Representation (`Document.frontmatter: Option<Metadata>`, raw/span semantics) → Task 1 (field) + Task 2 (`frontmatter` builds the `Metadata`).
- D1 lenient fence grammar (`={3,}`, indent, independent fences; `=== x`/`==` not fences) → Task 2 (`fence_marker` + `four_or_more_equals`, `indented_fence`, `two_equals_is_not_a_fence` tests).
- D2 non-leading fence = error → Task 4 (`at_content_line` guard + `misplaced_fence` + three tests).
- D3 blank-line/EOF after close → Task 2 (`eof_immediately_after_close`) + Task 3 (`content_immediately_after_close_fence_is_an_error`).
- D4 unterminated = error → Task 2 (`unterminated_block_is_an_error`) + Task 3 (`unterminated_frontmatter_is_an_error`).
- Leading trivia / no-frontmatter / frontmatter-only / CRLF / non-YAML → Task 2 + Task 3 tests.
- Regression after `frontmatter: None` churn → Task 1 Step 5 + Task 5 Step 3.

**Placeholder scan:** none — every code step shows complete code and exact commands.

**Type consistency:** `fence_marker` returns `Range<usize>`; `frontmatter` returns `ModalResult<Metadata>`; `fence_line` returns `ModalResult<()>`; `misplaced_fence` returns `ModalResult<Block>`. `Document { blocks, frontmatter }` matches the field added in Task 1. Import names (`opt`, `peek`, `ContextError`, `ErrMode`, `fence_line`, `frontmatter`, `inline_ws`, `blank_line`) are introduced where first used.

## Blast-radius

Lightweight (skipping the formal tool — the change set is small, fully enumerated, and not cross-cutting):
- **Changed:** `crates/argdown-core/src/ast.rs` (1 additive field), `crates/argdown-parser/src/{frontmatter.rs (new), lib.rs, text.rs}`.
- **Downstream:** `argdown-mcp` depends on `argdown_core::Document` but only `Debug`-prints it → no break (new field prints automatically). No other crate constructs `Document` literals outside the four updated in Task 1.
- **Risk:** low. The only behavior change to pre-existing inputs is that a bare `===`/`====` line (which previously parsed as statement text) is now a hard error per D2 — intentional and covered by tests.
