# Layer B Statements (B3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the third Layer-B slice — a new `argdown_model::statements` module whose `build_statements(&Document) -> Statements` turns the flat `Block::Statement` AST into an equivalence-class model (one entity per title that appears in the document), with a block→entity assignment and a redefinition-conflict list surfaced as data.

**Architecture:** A new module `crates/argdown-model/src/statements.rs` in the existing `argdown-model` crate (B1 and B2's home). Picked up by the existing `members = ["crates/*"]` workspace glob. **No new external dependency** — B3 reuses B2's `parse_metadata` and re-exported `Value` for `canonical_metadata`; the only new `std` type it touches is `HashMap` (for the title→id map). The public surface is one pure, total function plus four types — the B1-parallel pattern (focused module, public re-exports, `argdown-mcp` untouched). Pure and total (no `Result`) — the strictness ("first definition wins; later definitions are conflicts") is data on `Statements::conflicts`, not a failure mode.

**Tech Stack:** Rust (edition 2024, stable toolchain). No new external deps. Tests use `argdown-parser` (dev-dependency) to build `Document` inputs from real Argdown (B1/B2-parallel pattern).

**Spec:** `docs/snowball/specs/2026-06-05-layer-b-statements-design.md`

**Branch:** Commit directly to `main` — consistent with the project convention; B3 is purely additive (a new module in an existing crate; the parser/core/MCP stays untouched) and does not bump the workspace version, so the version-gated release workflow will not fire.

---

## File Structure

| File | Responsibility | Change |
| ---- | -------------- | ------ |
| `crates/argdown-model/src/statements.rs` | `StatementId` / `Statement` / `StatementConflict` / `Statements` types, `build_statements`, tests | Create |
| `crates/argdown-model/src/lib.rs` | Crate root: module decl + public re-exports | Modify: add `mod statements; pub use statements::{...};` |

No `Cargo.toml` changes — B3 adds no new external dep.

---

### Task 1: Scaffold `statements.rs` with the types and a stub `build_statements`

**Files:**
- Create: `crates/argdown-model/src/statements.rs`
- Modify: `crates/argdown-model/src/lib.rs` — module decl + re-exports

- [ ] **Step 1: Create `statements.rs` with the four types and a stub function**

Create `crates/argdown-model/src/statements.rs`:

```rust
//! Statement equivalence-class model (Layer B, slice B3).
//!
//! Turns the flat `Block::Statement` AST into a registry of unique
//! statement entities (one per title that appears in the document) plus a
//! block→entity assignment. Pure and total — strictness ("first definition
//! wins; later definitions are conflicts") is surfaced as data on
//! [`Statements::conflicts`], not as a `Result` failure. Inline statement
//! mentions (`StatementMention` in inlines) are not entities; B3 handles
//! block-level statements only.

use argdown_core::{Document, Span};

pub use crate::metadata::Value;

/// Stable, source-order id; indexes `Statements::statements`.
///
/// Stable within a single parse only (the source is re-parsed fresh each
/// time); not designed to survive edits. Matches `SectionId` from B1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatementId(pub usize);

/// One statement entity in the equivalence class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub id: StatementId,
    /// Always `Some` — only titled statements form entities. Plain-text
    /// (untitled) statements are not in the model.
    pub title: String,
    /// First definition's text, or `None` if the entity is referenced
    /// but never defined in this document.
    pub canonical_text: Option<String>,
    /// First definition's metadata, parsed via B2's `parse_metadata`;
    /// `None` if no definition, the definition had no metadata block, or
    /// `parse_metadata` returned an error (B3 is total — B2 errors are
    /// absorbed as "no parsed metadata" rather than propagated).
    pub canonical_metadata: Option<Value>,
}

/// A redefinition conflict: a title was defined more than once. Surfaced
/// as data on `Statements::conflicts`, not as a `Result` failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementConflict {
    pub title: String,
    /// Source span of the first (canonical) definition.
    pub canonical_span: Span,
    /// Source spans of every later (conflicting) definition, in source
    /// order.
    pub conflicting_spans: Vec<Span>,
}

/// The B3 output: a flat statement arena, a block→statement assignment,
/// and a list of redefinition conflicts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Statements {
    /// Flat arena. `StatementId(i)` indexes `statements[i]`. Source order
    /// of first occurrence (definition or reference — whichever comes
    /// first in the document).
    pub statements: Vec<Statement>,
    /// Index-aligned with `document.blocks`: the statement entity for each
    /// titled-statement block, or `None` for plain-text statements and
    /// non-statement blocks (Heading, Argument, Relation, Pcs).
    pub block_statements: Vec<Option<StatementId>>,
    /// Redefinition conflicts found while walking the document, in source
    /// order (sorted by the order their title first appeared).
    pub conflicts: Vec<StatementConflict>,
}

/// Build the statement equivalence-class model for a parsed document.
///
/// Single pass over `document.blocks`, maintaining a `title → StatementId`
/// map. An entity is created on first occurrence of a title (definition
/// or reference — whichever comes first); the canonical is filled in on
/// the first definition; later definitions append to a per-title
/// `StatementConflict`. Plain-text statements and non-statement blocks
/// push `None` to `block_statements`.
pub fn build_statements(_document: &Document) -> Statements {
    Statements::default()
}
```

- [ ] **Step 2: Wire the module into `lib.rs`**

In `crates/argdown-model/src/lib.rs`, add the module declaration and re-exports. The current `lib.rs` is:

```rust
//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly, B2 provides
//! metadata parsing.

mod metadata;
mod sections;

pub use metadata::{MetadataError, Value, parse_metadata};
pub use sections::{Section, SectionId, Sections, build_sections};
```

Replace it with (adding `mod statements;` and a re-export line — leave the existing `metadata` and `sections` blocks and their comments untouched):

```rust
//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly, B2 provides
//! metadata parsing, B3 provides statement equivalence classes.

mod metadata;
mod sections;
mod statements;

pub use metadata::{MetadataError, Value, parse_metadata};
pub use sections::{Section, SectionId, Sections, build_sections};
pub use statements::{Statement, StatementConflict, StatementId, Statements, build_statements};
```

- [ ] **Step 3: Build and run the full CI gate**

Run: `cargo fmt --all`
Then: `cargo clippy --workspace --all-targets --locked -- -D warnings`
Then: `cargo build --workspace --locked`
Then: `cargo test --workspace --locked`

Expected:
- `cargo fmt` reformats nothing (or only the new files, no diff).
- `cargo clippy` and `cargo build` succeed; `_document` is intentionally unused in the stub (the leading underscore keeps clippy quiet).
- `cargo test` shows the previous test counts unchanged: 145 passing (3 core + 22 model — 9 B1 sections + 13 B2 metadata — + 120 parser + 0 doc tests × 4).

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-model/src/statements.rs crates/argdown-model/src/lib.rs
git commit -m "feat: scaffold argdown-model::statements with equivalence types (B3)"
```

---

### Task 2: Implement `build_statements` (TDD)

**Files:**
- Modify: `crates/argdown-model/src/statements.rs`

- [ ] **Step 1: Add a failing test for the happy path (single titled definition)**

In `crates/argdown-model/src/statements.rs`, add a `#[cfg(test)] mod tests` block at the end of the file. Use the function-stub identifier `_document` (currently unused) as the test's input — the test will panic against the stub, which is the expected red:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use argdown_parser::parse;

    #[test]
    fn single_titled_definition_creates_one_entity() {
        let doc = parse("[A]: claim").unwrap();
        let s = build_statements(&doc);

        assert_eq!(s.statements.len(), 1);
        assert_eq!(s.statements[0].id, StatementId(0));
        assert_eq!(s.statements[0].title, "A");
        assert_eq!(s.statements[0].canonical_text.as_deref(), Some("claim"));
        assert_eq!(s.statements[0].canonical_metadata, None);
        assert!(s.conflicts.is_empty());
        assert_eq!(s.block_statements, vec![Some(StatementId(0))]);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p argdown-model single_titled_definition_creates_one_entity`
Expected: FAIL — the stub returns `Statements::default()`, so `s.statements.len()` is 0, not 1, and the first assertion panics.

- [ ] **Step 3: Replace the stub with the single-pass implementation**

In `crates/argdown-model/src/statements.rs`, replace the body of `build_statements` (and remove the leading underscore from the parameter — it's now used). The doc comment above the function stays as it is. Replace the function definition with:

```rust
pub fn build_statements(document: &Document) -> Statements {
    use std::collections::HashMap;

    let mut statements: Vec<Statement> = Vec::new();
    let mut by_title: HashMap<String, StatementId> = HashMap::new();
    // Conflicts keyed by title; drained and sorted at the end so the
    // output is in source order of the title's first appearance.
    let mut conflict_map: HashMap<String, StatementConflict> = HashMap::new();
    let mut block_statements: Vec<Option<StatementId>> =
        Vec::with_capacity(document.blocks.len());

    for block in &document.blocks {
        // 1. Resolve the block's id (if any).
        let id = match block {
            Block::Statement(s) => s.title.as_ref().map(|title| {
                *by_title.entry(title.clone()).or_insert_with(|| {
                    let id = StatementId(statements.len());
                    statements.push(Statement {
                        id,
                        title: title.clone(),
                        canonical_text: None,
                        canonical_metadata: None,
                    });
                    id
                })
            }),
            _ => None,
        };

        // 2. For a definition, fill in canonical on first occurrence and
        //    record a conflict on a redefinition.
        if let (Block::Statement(s), Some(id)) = (block, id) {
            if !s.is_reference {
                let title = s
                    .title
                    .as_ref()
                    .expect("a statement block with a resolved id has a title");
                let entry = &mut statements[id.0];
                if entry.canonical_text.is_none() {
                    entry.canonical_text = Some(s.text.clone());
                    entry.canonical_metadata = s
                        .metadata
                        .as_ref()
                        .map(crate::metadata::parse_metadata)
                        .transpose()
                        .ok()
                        .flatten();
                    // First definition is the canonical — record its span
                    // in any conflict entry (created empty if no conflict
                    // has been recorded for this title yet).
                    let entry = conflict_map
                        .entry(title.clone())
                        .or_insert_with(|| StatementConflict {
                            title: title.clone(),
                            canonical_span: s.span,
                            conflicting_spans: Vec::new(),
                        });
                    entry.canonical_span = s.span;
                } else {
                    // Already defined — this is a redefinition conflict.
                    let entry = conflict_map
                        .entry(title.clone())
                        .or_insert_with(|| StatementConflict {
                            title: title.clone(),
                            canonical_span: s.span,
                            conflicting_spans: Vec::new(),
                        });
                    entry.conflicting_spans.push(s.span);
                }
            }
        }

        // 3. Record the block→entity mapping.
        block_statements.push(id);
    }

    // Drain conflicts in source order (by first appearance of the title).
    let mut conflicts: Vec<StatementConflict> = conflict_map.into_values().collect();
    conflicts.sort_by_key(|c| {
        statements
            .iter()
            .position(|s| s.title == c.title)
            .unwrap_or(0)
    });

    Statements {
        statements,
        block_statements,
        conflicts,
    }
}
```

Also add the import at the top of the file (next to the existing `use argdown_core::{Document, Span};`):

```rust
use argdown_core::{Block, Document, Span};
```

(Note: the `_document` underscore is removed from the function signature in the replacement above.)

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p argdown-model single_titled_definition_creates_one_entity`
Expected: PASS.

- [ ] **Step 5: Format, lint, and run all crate tests**

Run: `cargo fmt --all`
Then: `cargo clippy --workspace --all-targets --locked -- -D warnings`
Then: `cargo test -p argdown-model`
Expected: `cargo fmt` makes no changes; `cargo clippy` is clean; `cargo test` reports `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (the previous 22 tests plus the new `single_titled_definition_creates_one_entity`).

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-model/src/statements.rs
git commit -m "feat: implement build_statements single-pass equivalence model (B3)"
```

---

### Task 3: Add coverage tests (empty, references, redefinitions, plain text, metadata, ordering)

**Files:**
- Modify: `crates/argdown-model/src/statements.rs` — extend `mod tests`

- [ ] **Step 1: Add the 12 coverage tests**

In `crates/argdown-model/src/statements.rs`, extend the `mod tests` block. The existing `single_titled_definition_creates_one_entity` test stays as it is. Append the following tests inside the same `mod tests { ... }` block (after the closing `}` of `single_titled_definition_creates_one_entity`):

```rust
    #[test]
    fn empty_document_has_no_statements() {
        let doc = parse("").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s, Statements::default());
    }

    #[test]
    fn single_titled_reference_has_no_canonical() {
        let doc = parse("[A]").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 1);
        assert_eq!(s.statements[0].title, "A");
        assert_eq!(s.statements[0].canonical_text, None);
        assert!(s.conflicts.is_empty());
        assert_eq!(s.block_statements, vec![Some(StatementId(0))]);
    }

    #[test]
    fn definition_then_reference_share_one_entity() {
        let doc = parse("[A]: claim\n\n[A]").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 1);
        assert_eq!(s.statements[0].canonical_text.as_deref(), Some("claim"));
        assert_eq!(
            s.block_statements,
            vec![Some(StatementId(0)), Some(StatementId(0))]
        );
        assert!(s.conflicts.is_empty());
    }

    #[test]
    fn reference_then_definition_fills_canonical_later() {
        let doc = parse("[A]\n\n[A]: claim").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 1);
        // The reference created the entity; the later definition filled canonical.
        assert_eq!(s.statements[0].canonical_text.as_deref(), Some("claim"));
        assert_eq!(
            s.block_statements,
            vec![Some(StatementId(0)), Some(StatementId(0))]
        );
        assert!(s.conflicts.is_empty());
    }

    #[test]
    fn redefinition_records_a_conflict() {
        let doc = parse("[A]: claim1\n\n[A]: claim2").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 1);
        // First definition wins.
        assert_eq!(s.statements[0].canonical_text.as_deref(), Some("claim1"));
        assert_eq!(s.conflicts.len(), 1);
        assert_eq!(s.conflicts[0].title, "A");
        assert_eq!(s.conflicts[0].conflicting_spans.len(), 1);
    }

    #[test]
    fn three_distinct_titles_create_three_entities_in_source_order() {
        let doc = parse("[A]: one\n\n[B]: two\n\n[C]: three").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 3);
        assert_eq!(s.statements[0].title, "A");
        assert_eq!(s.statements[1].title, "B");
        assert_eq!(s.statements[2].title, "C");
        assert_eq!(
            s.block_statements,
            vec![
                Some(StatementId(0)),
                Some(StatementId(1)),
                Some(StatementId(2)),
            ]
        );
        assert!(s.conflicts.is_empty());
    }

    #[test]
    fn plain_text_statement_is_not_an_entity() {
        let doc = parse("just some text").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 0);
        assert_eq!(s.block_statements, vec![None]);
        assert!(s.conflicts.is_empty());
    }

    #[test]
    fn non_statement_blocks_have_no_statement_id() {
        // A heading and an argument definition — neither is a statement.
        let doc = parse("# heading\n\n<A>: desc\n\n> argument").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 0);
        assert_eq!(s.block_statements, vec![None, None, None]);
        assert!(s.conflicts.is_empty());
    }

    #[test]
    fn three_redefinitions_record_two_conflicting_spans() {
        let doc = parse("[A]: c1\n\n[A]: c2\n\n[A]: c3").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 1);
        assert_eq!(s.statements[0].canonical_text.as_deref(), Some("c1"));
        assert_eq!(s.conflicts.len(), 1);
        assert_eq!(s.conflicts[0].conflicting_spans.len(), 2);
    }

    #[test]
    fn canonical_metadata_is_parsed() {
        // Inline metadata on the definition: the parser captures it as
        // Statement.metadata; build_statements parses it via B2's
        // parse_metadata and stores the result as canonical_metadata.
        let doc = parse("[A]: claim { key: value }").unwrap();
        let s = build_statements(&doc);
        let meta = s.statements[0]
            .canonical_metadata
            .as_ref()
            .expect("definition had metadata");
        let Value::Mapping(map) = meta else {
            panic!("expected Value::Mapping, got {meta:?}");
        };
        assert!(map.contains_key("key"));
    }

    #[test]
    fn parser_normalizes_titles_by_trimming() {
        // The parser's `statement_title` already trims whitespace; B3
        // doesn't re-normalize, so the trim is inherited. This test
        // documents that expectation.
        let doc = parse("[ A ]: claim").unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 1);
        assert_eq!(s.statements[0].title, "A");
    }

    #[test]
    fn conflicts_are_sorted_in_source_order_of_title_first_appearance() {
        // Titles X, Y, Z appear in order X, Y, Z. Each is redefined in the
        // same order (X at block 3, Y at block 5, Z at block 7). Conflicts
        // should come out in source order: X, Y, Z (the order the titles
        // first appeared, not the order the redefinitions happened).
        let doc = parse(
            "[X]: x1\n\n[Y]: y1\n\n[Z]: z1\n\n[X]: x2\n\n[Y]: y2\n\n[Z]: z2",
        )
        .unwrap();
        let s = build_statements(&doc);
        assert_eq!(s.statements.len(), 3);
        assert_eq!(s.conflicts.len(), 3);
        assert_eq!(s.conflicts[0].title, "X");
        assert_eq!(s.conflicts[1].title, "Y");
        assert_eq!(s.conflicts[2].title, "Z");
    }
```

- [ ] **Step 2: Run all crate tests**

Run: `cargo test -p argdown-model`
Expected: `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (the 9 B1 sections + the 13 B2 metadata + the 1 B3 happy-path + the 12 B3 coverage = 35).

If any test fails, the implementation has a bug — fix it before continuing. The most likely failure modes:
- `three_redefinitions_record_two_conflicting_spans` failing means the conflict accumulation isn't tracking per-title state correctly.
- `conflicts_are_sorted_in_source_order_of_title_first_appearance` failing means the final sort is missing or wrong.
- `canonical_metadata_is_parsed` failing means the B2 integration (`s.metadata.as_ref().map(parse_metadata)`) isn't wired right — confirm `crate::metadata::parse_metadata` is the right path (it should be, since `lib.rs` re-exports it).

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-model/src/statements.rs
git commit -m "test: cover empty, references, conflicts, plain text, metadata, ordering (B3)"
```

---

### Task 4: Final CI gate and clean tree

**Files:**
- Modify: (no source files; the gate itself)

- [ ] **Step 1: Run the full CI gate exactly as `ci.yml` runs it**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
```

Expected:
- `fmt --check` exits 0 with no output.
- `clippy` is clean (no warnings, no errors).
- `build` succeeds.
- `test` ends with `test result: ok.` for every crate. Total: 3 core + 35 model (9 B1 + 13 B2 + 13 B3) + 120 parser = 158.

These mirror the CI `fmt` and `check` jobs, so a clean local pass predicts a green push.

- [ ] **Step 2: Confirm a clean working tree**

Run: `git status --short`
Expected: only `docs/snowball/decisions/observations.jsonl` modified (snowball-hook auto-append) and `.idea/` untracked (gitignored). No source-tree modifications.

- [ ] **Step 3: Commit any hook-appended observations**

```bash
git add docs/snowball/decisions/observations.jsonl
# only commit if the diff is non-empty; otherwise skip
git diff --cached --quiet || git commit -m "chore: snowball observations from B3 implementation session"
```

(If `git diff --cached --quiet` returns 0, there's nothing to commit — skip the commit and move on.)

---

## Self-Review

**Spec coverage:**
- `StatementId`, `Statement`, `StatementConflict`, `Statements` types exactly as specified → Task 1 Step 1. ✓
- `build_statements(&Document) -> Statements` (pure, total, no `Result`) → Task 1 Step 1 + Task 2 Step 3. ✓
- Single-pass algorithm with `HashMap<title, StatementId>` for entity creation, canonical-on-first-definition, conflict-on-redefinition → Task 2 Step 3. ✓
- Plain-text statements and non-statement blocks push `None` to `block_statements` → Task 2 Step 3 (the `_ => None` arm of the `match block` + the `id` always pushed at the end) + Task 3 Step 1 tests 8 and 9. ✓
- B2's `parse_metadata` consumed internally; B2 errors absorbed as `canonical_metadata: None` → Task 2 Step 3 (`.transpose().ok().flatten()`) + Task 3 Step 1 test 11. ✓
- Conflicts drained in source order (by first appearance of the title) → Task 2 Step 3 (the final `sort_by_key` block) + Task 3 Step 1 test 13. ✓
- All 13 spec-mandated tests (empty, single def, single ref, def+ref, ref+def, redefinition, three titles, plain text, non-statement blocks, three redefinitions, canonical metadata, title normalization, conflict sort order) → Task 2 Step 1 (test 2) + Task 3 Step 1 (tests 1, 3–13) = 13 tests. ✓
- TDD discipline: Task 2 has a failing-test-first step (Step 2 verifies failure, Step 4 verifies pass) → matches the spec's TDD note. ✓
- `argdown-mcp` not modified → no step touches `argdown-mcp`. ✓
- No `Cargo.toml` changes (no new external dep) → no step touches `Cargo.toml`. ✓
- Out-of-scope items absent: no `Model` aggregate, no inline-mention entities, no per-statement occurrence list, no `BlockId` concept, no "referenced but never defined" detection, no references-to-undefined detection, no per-section statement lists, no typed accessors on `Statements` → respected throughout. ✓

**Placeholder scan:** No "TBD", "TODO", "implement later", "fill in details", or "similar to Task N". All 12 test bodies are spelled out in Task 3 Step 1; the implementation body is spelled out in Task 2 Step 3. ✓

**Type/name consistency:** `StatementId`, `Statement`, `StatementConflict`, `Statements`, `build_statements` are used identically across all tasks. The function signature `build_statements(document: &Document) -> Statements` is identical in Task 1 (with `_document` and stub body) and Task 2 (with `document` and full body). The `use` statement at the top of `statements.rs` (`use argdown_core::{Block, Document, Span};`) is added in Task 2 Step 3 (the stub doesn't need `Block`; the implementation does). The `use std::collections::HashMap;` is local to the function body to keep the import scope tight. The re-exports in `lib.rs` (`Statement`, `StatementConflict`, `StatementId`, `Statements`, `build_statements`) match the types defined in `statements.rs`. ✓

---

## Summary

B3 is the third slice of Layer B. The plan scaffolds the new `statements` module with the four types and a stub function, implements the single-pass `build_statements` algorithm via TDD, adds 12 coverage tests (empty, single reference, def+ref, ref+def, redefinition, three titles, plain text, non-statement blocks, three redefinitions, canonical metadata, title normalization, conflict sort order), and closes with the same CI gate B1 and B2 used. Four tasks, 16 steps; the implementation is one function with a clear single-pass algorithm; the bulk of the work is the test surface (13 tests in `argdown-model`'s new statements module, on top of the 9 B1 sections tests and the 13 B2 metadata tests). argdown-mcp is not modified, and no new external dependencies are added.
