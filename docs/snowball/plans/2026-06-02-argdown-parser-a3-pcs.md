# Argdown Parser — A3 (Premise-Conclusion Structures) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Parse premise-conclusion structures (PCS) — numbered statement lines, inference lines, and interspersed relations — into a flat, source-order `Block::Pcs` of form-tagged items.

**Architecture:** Additive AST in `argdown-core` (`Block::Pcs(Pcs)`, `Pcs { items }`, `PcsItem`); a new `pcs.rs` parser module whose block parser parses a leading numbered statement (the commit point) then loops over items {numbered statement, inference line, relation}; a new `pcs` dispatch arm; and one `pcs_marker` continuation guard in `text.rs`. Roles, inference→conclusion binding, and relation association are deferred to Layer B (not parsed here).

**Tech Stack:** Rust, `winnow` 1.x parser combinators (`LocatingSlice` for byte spans), Cargo workspace (`argdown-core`, `argdown-parser`, `argdown-mcp`).

**Spec:** `docs/snowball/specs/2026-06-02-argdown-parser-a3-pcs-design.md`

**Conventions (follow exactly):**
- TDD: write the failing test, run it, watch it fail for the right reason, then write the minimal code to pass.
- Reuse the A2a `statement` parser and the A2b `relation` parser verbatim for targets — do not re-implement them.
- All new AST types derive `Debug, Clone, PartialEq, Eq`. Do not modify `Statement`, `Argument`, or `Relation`.
- Run `cargo test -p <crate>` after each step; keep all prior A1/A2a/A2b tests green.
- Tests live in the existing `#[cfg(test)] mod tests` in `crates/argdown-parser/src/lib.rs`, alongside the relation tests.

---

### Task 1: Add PCS AST types (`argdown-core`)

Pure additive data declarations. There is no behavior to TDD here; these types are *enabling declarations* for the parser tests in Task 2+. Verify with a build, not a unit test.

**Files:**
- Modify: `crates/argdown-core/src/ast.rs`
- Modify: `crates/argdown-core/src/lib.rs`

- [ ] **Step 1: Add the `Pcs` variant to `Block`**

In `crates/argdown-core/src/ast.rs`, change the `Block` enum:

```rust
/// A top-level block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(Heading),
    Statement(Statement),
    Argument(Argument),
    Relation(Relation),
    Pcs(Pcs),
}
```

- [ ] **Step 2: Add the PCS types**

In `crates/argdown-core/src/ast.rs`, immediately after the `Relation`/`RelationTarget` types and before the `#[cfg(test)] mod tests` block, add:

```rust
/// A premise-conclusion structure: a flat, source-order sequence of items.
/// Role assignment (premise/conclusion), inference→conclusion binding, and
/// relation association are Layer B's job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcs {
    pub items: Vec<PcsItem>,
    /// First item span start → last item span end.
    pub span: Span,
}

/// One line of a PCS, tagged by form (not role).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PcsItem {
    /// `(n) <statement>` — content reuses the statement forms.
    Statement {
        number: usize,
        statement: Statement,
        /// The `(` of the marker → statement content end.
        span: Span,
    },
    /// `----` (bare → empty rules) or `-- Rule, Rule --` (ruled).
    Inference { rules: Vec<String>, span: Span },
    /// An interspersed relation line, reusing the relation form (with indent).
    Relation(Relation),
}
```

- [ ] **Step 3: Export the new types**

In `crates/argdown-core/src/lib.rs`, extend the `pub use ast::{...}` list to include `Pcs` and `PcsItem`:

```rust
pub use ast::{
    Argument, Block, Document, Heading, Pcs, PcsItem, Relation, RelationDirection,
    RelationOperator, RelationTarget, Span, Statement,
};
pub use error::Error;
```

- [ ] **Step 4: Verify the workspace builds**

Run: `cargo build --workspace`
Expected: builds cleanly (a `failed to auto-clean cache data` warning from the global cargo cache is unrelated and fine). `argdown-mcp` is unaffected — it `Debug`-prints `Document`, it does not `match` on `Block`.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-core/src/ast.rs crates/argdown-core/src/lib.rs
git commit -m "feat: add PCS AST types (A3)"
```

---

### Task 2: Numbered statements + dispatch (`pcs.rs` skeleton)

Drive the core PCS block parser and the dispatch arm into existence with the simplest PCS: one numbered statement, then two.

**Files:**
- Create: `crates/argdown-parser/src/pcs.rs`
- Modify: `crates/argdown-parser/src/lib.rs` (add `mod pcs;`, `use pcs::pcs;`, dispatch arm, test imports)
- Test: `crates/argdown-parser/src/lib.rs` (tests module)

- [ ] **Step 1: Extend the test-module imports**

In `crates/argdown-parser/src/lib.rs`, update the `use argdown_core::{...}` line inside `mod tests` to add `Pcs` and `PcsItem`:

```rust
    use argdown_core::{
        Argument, Heading, Pcs, PcsItem, Relation, RelationDirection, RelationOperator,
        RelationTarget, Span, Statement,
    };
```

- [ ] **Step 2: Write the failing tests**

Add to the `mod tests` block in `crates/argdown-parser/src/lib.rs`:

```rust
    /// Extract the single PCS a source parses to, panicking otherwise.
    fn only_pcs(src: &str) -> Pcs {
        match parse(src).unwrap().blocks.as_slice() {
            [Block::Pcs(p)] => p.clone(),
            other => panic!("{src:?} did not parse as a single PCS: {other:?}"),
        }
    }

    #[test]
    fn pcs_single_numbered_statement() {
        let pcs = only_pcs("(1) a");
        assert_eq!(pcs.items.len(), 1);
        match &pcs.items[0] {
            PcsItem::Statement {
                number, statement, ..
            } => {
                assert_eq!(*number, 1);
                assert_eq!(statement.text, "a");
                assert_eq!(statement.title, None);
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn pcs_two_numbered_statements() {
        let pcs = only_pcs("(1) a\n(2) b");
        let numbers: Vec<usize> = pcs
            .items
            .iter()
            .map(|item| match item {
                PcsItem::Statement { number, .. } => *number,
                other => panic!("expected statement items, got {other:?}"),
            })
            .collect();
        assert_eq!(numbers, vec![1, 2]);
    }
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser pcs_single_numbered_statement pcs_two_numbered_statements`
Expected: FAIL — `(1) a` currently parses as a plain `Statement` (text `"(1) a"`), not a `Block::Pcs`.

- [ ] **Step 4: Create the `pcs.rs` parser**

Create `crates/argdown-parser/src/pcs.rs`:

```rust
//! Premise-conclusion structures (PCS): numbered statement lines, inference
//! lines, and interspersed relations, emitted as a flat sequence of form-tagged
//! items in source order. Role assignment, inference→conclusion binding, and
//! relation association are Layer B's job.

use argdown_core::{Pcs, PcsItem, Span};
use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::digit1;
use winnow::combinator::{cut_err, delimited, opt};

use crate::Input;
use crate::statement::statement;
use crate::text::inline_ws;

/// Parse a PCS block: a leading numbered statement (which commits to a PCS),
/// then a run of items until a non-item line. The numbered marker `(n)` is what
/// distinguishes a PCS from a plain statement, so a line that is not a numbered
/// statement makes `pcs` backtrack and the dispatcher fall through.
pub(crate) fn pcs(input: &mut Input<'_>) -> ModalResult<Pcs> {
    let first = numbered_statement_item(input)?;
    let start = item_span_start(&first);
    let mut items = vec![first];
    while let Some(item) = opt(numbered_statement_item).parse_next(input)? {
        items.push(item);
    }
    let end = item_span_end(items.last().expect("pcs has at least one item"));
    Ok(Pcs {
        items,
        span: Span { start, end },
    })
}

/// `(n) <statement>` — the marker commits (so a bad/empty body is an error),
/// but a line that is not `( digits )` backtracks so dispatch can continue.
fn numbered_statement_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    inline_ws.parse_next(input)?;
    let (number, marker_span) = pcs_number.with_span().parse_next(input)?;
    inline_ws.parse_next(input)?;
    let stmt = cut_err(statement).parse_next(input)?;
    let span = Span {
        start: marker_span.start,
        end: stmt.span.end,
    };
    Ok(PcsItem::Statement {
        number,
        statement: stmt,
        span,
    })
}

/// `( digits )` → the numeric value. Backtracks if the line is not a numbered
/// marker (e.g. `(see note)`), so such lines fall through to a plain statement.
fn pcs_number(input: &mut Input<'_>) -> ModalResult<usize> {
    delimited('(', digit1, ')')
        .try_map(|s: &str| s.parse::<usize>())
        .parse_next(input)
}

fn item_span_start(item: &PcsItem) -> usize {
    match item {
        PcsItem::Statement { span, .. } | PcsItem::Inference { span, .. } => span.start,
        PcsItem::Relation(relation) => relation.span.start,
    }
}

fn item_span_end(item: &PcsItem) -> usize {
    match item {
        PcsItem::Statement { span, .. } | PcsItem::Inference { span, .. } => span.end,
        PcsItem::Relation(relation) => relation.span.end,
    }
}
```

- [ ] **Step 5: Wire `pcs` into the parser module and dispatch**

In `crates/argdown-parser/src/lib.rs`, add the module declaration next to the others:

```rust
mod argument;
mod heading;
mod pcs;
mod relation;
mod statement;
mod text;
mod trivia;
```

Add the import next to the others:

```rust
use pcs::pcs;
```

Add the `pcs` arm to `block`, after `relation` and before `argument`:

```rust
fn block(input: &mut Input<'_>) -> ModalResult<Block> {
    alt((
        heading.map(Block::Heading),
        relation.map(Block::Relation),
        pcs.map(Block::Pcs),
        argument.map(Block::Argument),
        statement.map(Block::Statement),
    ))
    .parse_next(input)
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — the two new tests pass and all prior tests stay green.

- [ ] **Step 7: Commit**

```bash
git add crates/argdown-parser/src/pcs.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: parse numbered PCS statements (A3)"
```

---

### Task 3: Bare inference line

Add the bare divider (`-{4,}`) as a PCS item, and the `---`-is-an-error rule.

**Files:**
- Modify: `crates/argdown-parser/src/pcs.rs`
- Test: `crates/argdown-parser/src/lib.rs` (tests module)

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block:

```rust
    #[test]
    fn pcs_bare_inference_line() {
        let pcs = only_pcs("(1) a\n(2) b\n----\n(3) c");
        assert_eq!(pcs.items.len(), 4);
        match &pcs.items[2] {
            PcsItem::Inference { rules, .. } => assert!(rules.is_empty()),
            other => panic!("expected an inference item at index 2, got {other:?}"),
        }
        // Statements either side keep their numbers.
        assert!(matches!(&pcs.items[1], PcsItem::Statement { number: 2, .. }));
        assert!(matches!(&pcs.items[3], PcsItem::Statement { number: 3, .. }));
    }

    #[test]
    fn pcs_bare_divider_allows_five_or_more_dashes() {
        let pcs = only_pcs("(1) a\n-----\n(2) b");
        assert!(matches!(&pcs.items[1], PcsItem::Inference { rules, .. } if rules.is_empty()));
    }

    #[test]
    fn pcs_three_dash_divider_is_an_error() {
        assert!(parse("(1) a\n---\n(2) b").is_err());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser pcs_bare_inference pcs_bare_divider pcs_three_dash`
Expected: FAIL — the inference line is not yet an item, so `----` ends the PCS after `(2) b` (the bare tests fail), and `---` does not error.

- [ ] **Step 3: Add the inference parser and the loop branch**

In `crates/argdown-parser/src/pcs.rs`, update the imports:

```rust
use winnow::ascii::{digit1, line_ending};
use winnow::combinator::{alt, cut_err, delimited, eof, opt, peek, preceded};
use winnow::token::{take_till, take_while};
```

Add `inference_item` after `numbered_statement_item`:

```rust
/// A bare divider (`-{4,}`) or a ruled divider (`-- Rule, Rule --`). A line
/// starting with `--` commits to an inference line; a malformed one (e.g. `---`,
/// or a ruled opener with no closing `--`) is an error.
fn inference_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    inline_ws.parse_next(input)?;
    peek("--").parse_next(input)?;
    let (rules, span) = cut_err(inference_rules).with_span().parse_next(input)?;
    opt(line_ending).parse_next(input)?;
    Ok(PcsItem::Inference { rules, span })
}

/// Classify an inference line already known to start with `--`.
fn inference_rules(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    alt((bare_divider, ruled_divider)).parse_next(input)
}

/// `-{4,}` followed by only trailing whitespace → no rules.
fn bare_divider(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    (
        take_while(4.., '-'),
        inline_ws,
        peek(alt((line_ending.void(), eof.void()))),
    )
        .map(|_| Vec::new())
        .parse_next(input)
}

/// `-- <content> --` on a single line → content split on commas into trimmed
/// rule names. Content is bounded to the current line (`take_till` stops at a
/// line ending), so a malformed line with no closing `--` fails here and
/// `inference_item`'s `cut_err` turns it into a hard error — rather than scanning
/// ahead to a later divider on another line.
fn ruled_divider(input: &mut Input<'_>) -> ModalResult<Vec<String>> {
    preceded("--", take_till(0.., ['\r', '\n']))
        .verify_map(|rest: &str| {
            let inner = rest.trim_end().strip_suffix("--")?;
            Some(
                inner
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect(),
            )
        })
        .parse_next(input)
}
```

Note: `delimited` is still imported because `pcs_number` (Task 2) uses it; `preceded` and `take_till` are new for `ruled_divider`.

Update the loop in `pcs` to try the inference item too (inference is tried before relations in later tasks; for now numbered + inference):

```rust
    while let Some(item) =
        opt(alt((numbered_statement_item, inference_item))).parse_next(input)?
    {
        items.push(item);
    }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — bare inference tests pass, `---` errors, all prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-parser/src/pcs.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: parse bare PCS inference lines (A3)"
```

---

### Task 4: Ruled inference line (rule names)

`ruled_divider` already exists from Task 3 (it is needed for the `alt`). This task adds the tests that exercise it and confirms the malformed-ruled error path.

**Files:**
- Test: `crates/argdown-parser/src/lib.rs` (tests module)

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block:

```rust
    fn inference_rules_of(src: &str, index: usize) -> Vec<String> {
        match &only_pcs(src).items[index] {
            PcsItem::Inference { rules, .. } => rules.clone(),
            other => panic!("expected an inference item at {index}, got {other:?}"),
        }
    }

    #[test]
    fn pcs_ruled_inference_single_rule() {
        assert_eq!(
            inference_rules_of("(1) a\n-- Modus Ponens --\n(2) b", 1),
            vec!["Modus Ponens".to_string()]
        );
    }

    #[test]
    fn pcs_ruled_inference_multiple_rules() {
        assert_eq!(
            inference_rules_of("(1) a\n-- Rule A, Rule B --\n(2) b", 1),
            vec!["Rule A".to_string(), "Rule B".to_string()]
        );
    }

    #[test]
    fn pcs_ruled_inference_without_closing_dashes_is_an_error() {
        assert!(parse("(1) a\n-- Modus Ponens\n(2) b").is_err());
    }

    #[test]
    fn pcs_multi_step_interleaved() {
        // premises -> bare inference -> intermediary -> premise -> ruled inference -> main
        let pcs = only_pcs("(1) a\n(2) b\n----\n(3) c\n(4) d\n-- R --\n(5) e");
        assert_eq!(pcs.items.len(), 7);
        assert!(matches!(&pcs.items[2], PcsItem::Inference { rules, .. } if rules.is_empty()));
        assert_eq!(inference_rules_of("(1) a\n(2) b\n----\n(3) c\n(4) d\n-- R --\n(5) e", 5),
            vec!["R".to_string()]);
        let numbers: Vec<usize> = pcs
            .items
            .iter()
            .filter_map(|item| match item {
                PcsItem::Statement { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(numbers, vec![1, 2, 3, 4, 5]);
    }
```

- [ ] **Step 2: Run the tests to verify they pass (or fail honestly)**

Run: `cargo test -p argdown-parser pcs_ruled_inference`
Expected: PASS — `ruled_divider` was implemented in Task 3 to make the `alt` type-check, so these tests confirm it. If `pcs_ruled_inference_single_rule` fails because `--` was consumed as something else, re-check that `inference_item` is in the loop's `alt` and that `peek("--")` precedes the `cut_err`.

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-parser/src/lib.rs
git commit -m "test: cover ruled PCS inference rule names (A3)"
```

---

### Task 5: Interspersed child relation

Add the relation branch to the item loop so a relation under a PCS statement is captured as a `PcsItem::Relation` (reusing the A2b parser).

**Files:**
- Modify: `crates/argdown-parser/src/pcs.rs`
- Test: `crates/argdown-parser/src/lib.rs` (tests module)

- [ ] **Step 1: Write the failing test**

Add to the `mod tests` block:

```rust
    #[test]
    fn pcs_interspersed_child_relation() {
        let pcs = only_pcs("(1) a\n  +> [X]\n----\n(2) b");
        assert_eq!(pcs.items.len(), 4);
        match &pcs.items[1] {
            PcsItem::Relation(relation) => {
                assert_eq!(relation.indent, 2);
                assert_eq!(relation.operator, RelationOperator::Support);
                assert_eq!(relation.direction, RelationDirection::Outbound);
                match &relation.target {
                    RelationTarget::Statement(s) => assert_eq!(s.title.as_deref(), Some("X")),
                    other => panic!("expected a statement target, got {other:?}"),
                }
            }
            other => panic!("expected a relation item at index 1, got {other:?}"),
        }
        assert!(matches!(&pcs.items[2], PcsItem::Inference { .. }));
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p argdown-parser pcs_interspersed_child_relation`
Expected: FAIL — the relation line is not yet an item, so the PCS ends at `(1) a` and the relation/inference become separate blocks (the `only_pcs` helper panics on >1 block).

- [ ] **Step 3: Add the relation branch**

In `crates/argdown-parser/src/pcs.rs`, add the relation import:

```rust
use crate::relation::relation;
```

Add `relation_item` after `inference_item`:

```rust
/// An interspersed relation line, reusing the relation parser.
fn relation_item(input: &mut Input<'_>) -> ModalResult<PcsItem> {
    relation.map(PcsItem::Relation).parse_next(input)
}
```

Update the loop's `alt` — inference **before** relation so `--` is an inference, not a `-` relation:

```rust
    while let Some(item) =
        opt(alt((numbered_statement_item, inference_item, relation_item))).parse_next(input)?
    {
        items.push(item);
    }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p argdown-parser`
Expected: PASS — the relation is captured as item 1 with indent 2; all prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-parser/src/pcs.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: capture interspersed relations as PCS items (A3)"
```

---

### Task 6: `pcs_marker` continuation guard, multi-line statements, boundaries

A numbered statement's content may span continuation lines, but it must stop at the next `(n)` marker. Add a `pcs_marker` guard to `at_content_line`.

**Files:**
- Modify: `crates/argdown-parser/src/text.rs`
- Test: `crates/argdown-parser/src/lib.rs` (tests module)

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block:

```rust
    #[test]
    fn pcs_numbered_statement_spans_continuation_lines() {
        let pcs = only_pcs("(1) one\n    two");
        assert_eq!(pcs.items.len(), 1);
        match &pcs.items[0] {
            PcsItem::Statement { statement, .. } => assert_eq!(statement.text, "one two"),
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn pcs_marker_ends_a_previous_statements_continuation() {
        // Without the guard, `(2) b` would be swallowed as continuation text of (1).
        let pcs = only_pcs("(1) one\ntwo\n(2) b");
        match &pcs.items[0] {
            PcsItem::Statement { number, statement, .. } => {
                assert_eq!(*number, 1);
                assert_eq!(statement.text, "one two");
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
        assert!(matches!(&pcs.items[1], PcsItem::Statement { number: 2, .. }));
    }

    #[test]
    fn pcs_ends_at_heading_and_reference() {
        let blocks = parse("(1) a\n# H").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Pcs(_)));
        assert!(matches!(&blocks[1], Block::Heading(_)));

        let blocks = parse("(1) a\n[X]").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Pcs(_)));
        assert!(matches!(&blocks[1], Block::Statement(s) if s.is_reference));
    }

    #[test]
    fn blank_line_separates_two_pcs_blocks() {
        let blocks = parse("(1) a\n\n(2) b").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Pcs(p) if p.items.len() == 1));
        assert!(matches!(&blocks[1], Block::Pcs(p) if p.items.len() == 1));
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p argdown-parser pcs_marker_ends pcs_numbered_statement_spans pcs_ends_at`
Expected: `pcs_marker_ends_a_previous_statements_continuation` FAILS — `(2) b` is swallowed into statement (1)'s text as `"one two (2) b"`, leaving a single item. (`pcs_numbered_statement_spans_continuation_lines` and `pcs_ends_at_heading_and_reference` may already pass — the `block_head`/`heading_marker` guards added in A1/A2a already stop continuation at `[`/`#`. They guard the boundary and should be kept.)

- [ ] **Step 3: Add the `pcs_marker` guard**

In `crates/argdown-parser/src/text.rs`, add the `digit1` import:

```rust
use winnow::ascii::{digit1, line_ending, till_line_ending};
```

Add `pcs_marker` next to `relation_marker`:

```rust
/// Match a line that begins (after indentation) with a numbered marker
/// `( digits )` — the start of a PCS statement. Lets a continuation line stop
/// before the next numbered statement instead of swallowing it as text.
pub(crate) fn pcs_marker(input: &mut Input<'_>) -> ModalResult<()> {
    (take_while(0.., [' ', '\t']), '(', digit1, ')')
        .void()
        .parse_next(input)
}
```

Add `not(pcs_marker)` to `at_content_line`:

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
    )
        .void()
        .parse_next(input)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p argdown-parser`
Expected: PASS — all three tests pass; all prior tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-parser/src/text.rs crates/argdown-parser/src/lib.rs
git commit -m "feat: stop PCS statement continuation at the next numbered marker (A3)"
```

---

### Task 7: Numbered-statement target forms + commit errors

Confirm numbered-statement content reuses every statement form, and that a committed-but-malformed numbered statement is a hard error.

**Files:**
- Test: `crates/argdown-parser/src/lib.rs` (tests module)

- [ ] **Step 1: Write the tests**

Add to the `mod tests` block:

```rust
    #[test]
    fn pcs_numbered_statement_target_forms() {
        // Definition target.
        match &only_pcs("(1) [P]: text").items[0] {
            PcsItem::Statement { statement, .. } => {
                assert_eq!(statement.title.as_deref(), Some("P"));
                assert_eq!(statement.text, "text");
                assert!(!statement.is_reference);
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
        // Reference target.
        match &only_pcs("(1) [P]").items[0] {
            PcsItem::Statement { statement, .. } => {
                assert_eq!(statement.title.as_deref(), Some("P"));
                assert!(statement.is_reference);
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
        // Plain target.
        match &only_pcs("(1) plain").items[0] {
            PcsItem::Statement { statement, .. } => {
                assert_eq!(statement.title, None);
                assert_eq!(statement.text, "plain");
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn pcs_numbered_marker_without_content_is_an_error() {
        // The marker commits; an empty body is a hard error, not a plain statement.
        assert!(parse("(1) a\n(2)").is_err());
    }

    #[test]
    fn pcs_text_after_reference_target_is_an_error() {
        assert!(parse("(1) [P] extra").is_err());
    }

    #[test]
    fn parenthesized_non_number_is_a_plain_statement() {
        // `(see note)` is not a numbered marker — it stays a plain statement.
        let blocks = parse("(see note)").unwrap().blocks;
        assert!(matches!(&blocks[0], Block::Statement(s) if s.text == "(see note)"));
    }
```

- [ ] **Step 2: Run the tests to verify behavior**

Run: `cargo test -p argdown-parser pcs_numbered_statement_target pcs_numbered_marker_without pcs_text_after_reference parenthesized_non_number`
Expected: PASS — targets reuse the statement parser (built in Task 2); the `cut_err(statement)` makes a committed empty/invalid body an error; `pcs_number`'s backtrack keeps `(see note)` a plain statement. If `pcs_numbered_marker_without_content_is_an_error` fails (returns Ok), confirm `numbered_statement_item` wraps `statement` in `cut_err`.

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-parser/src/lib.rs
git commit -m "test: cover PCS target forms and commit errors (A3)"
```

---

### Task 8: Full verification

**Files:** none (verification only).

- [ ] **Step 1: Run the whole workspace test suite**

Run: `cargo test --workspace`
Expected: all `argdown-core` and `argdown-parser` tests pass (every prior A1/A2a/A2b test plus the new PCS tests). 0 failures.

- [ ] **Step 2: Lint**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: exit 0, no warnings. (Ignore the unrelated `failed to auto-clean cache data` cargo registry message.)

- [ ] **Step 3: Format**

Run: `cargo fmt --all` then `cargo fmt --all -- --check`
Expected: `--check` exits 0 (clean).

- [ ] **Step 4: Commit any formatting changes**

```bash
git add -A
git commit -m "chore: cargo fmt after A3 PCS" || echo "nothing to format"
```

---

## Done criteria (from the spec)

- `cargo test` passes, including the new PCS tests and all prior tests.
- `parse()` emits `Block::Pcs` items with correct numbers, inference rule names, relation items, and spans, in source order; numbered-statement content reuses the statement forms and relations reuse the A2b form.
- Clean under `cargo clippy -- -D warnings`; canonical formatting.
- Deferred to later increments (do NOT implement here): premise/conclusion roles, inference→conclusion binding, relation association, argument↔PCS linking, equivalence classes (Layer B); `{…}` inference metadata (A5).
