# Layer B Arguments (B4a) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:test-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the argument half of Layer B slice B4 — a new
`argdown_model::arguments` module whose `build_arguments(&Document) ->
Arguments` turns the flat `Block::Argument` AST into an equivalence-class model
(one entity per `<Title>` appearing as a top-level block), with a block→entity
assignment and a redefinition-conflict list surfaced as data.

**Architecture:** New module `crates/argdown-model/src/arguments.rs` in the
existing `argdown-model` crate. Reuses B2's `parse_metadata`/`Value` and
`std::collections::HashMap`. **No new external dependency, no `Cargo.toml`
change.** Pure and total (no `Result`). A near-direct mirror of B3's
`statements.rs`.

**Spec:** `docs/snowball/specs/2026-06-06-layer-b-arguments-design.md`

**Branch:** Commit directly to `main` — B4a is purely additive (a new module in
an existing crate; no version bump), so the version-gated release workflow does
not fire.

---

## File Structure

| File | Responsibility | Change |
| ---- | -------------- | ------ |
| `crates/argdown-model/src/arguments.rs` | `ArgumentId` / `Argument` / `ArgumentConflict` / `Arguments` types, `build_arguments`, tests | Create |
| `crates/argdown-model/src/lib.rs` | module decl + public re-exports | Modify |

---

### Task 1: Scaffold `arguments.rs` + wire `lib.rs`

- [ ] Create `arguments.rs` with the four types (per spec) and a stub
  `pub fn build_arguments(_document: &Document) -> Arguments { Arguments::default() }`.
- [ ] In `lib.rs` add `mod arguments;` and
  `pub use arguments::{Argument, ArgumentConflict, ArgumentId, Arguments, build_arguments};`.
  Update the crate doc comment to mention B4a.
- [ ] Gate: `cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo build --workspace --locked`. Stub builds clean (`_document` underscore keeps clippy quiet).

### Task 2: Implement `build_arguments` (TDD)

- [ ] **RED:** add `single_argument_definition_creates_one_entity` (parses
  `<A>: desc`, asserts 1 entity, `canonical_description == Some("desc")`,
  `block_arguments == [Some(ArgumentId(0))]`, no conflicts). Run it; watch it
  fail against the stub.
- [ ] **GREEN:** replace the stub with the single-pass algorithm (mirror
  `build_statements`: `by_title` map, `canonical_spans` map, `conflict_map`;
  swap `Block::Argument`/`a.description`/`canonical_description`; resolve the
  entity unconditionally since `title` is a `String`). Remove the `_` from the
  parameter. Run; watch it pass.
- [ ] Gate (`fmt`, `clippy`, `cargo test -p argdown-model`).

### Task 3: Coverage tests (TDD)

- [ ] Add the remaining 11 tests from the spec's Testing section (empty,
  reference, def+ref, ref+def, redefinition, three titles, non-argument blocks,
  three redefinitions, canonical metadata, title trimming, conflicts sorted).
  Add them red-first where a behavior isn't yet proven; all should pass against
  the Task 2 implementation (it's complete) — any failure is a real bug, fix it.
- [ ] Gate: `cargo test -p argdown-model` → **47 model tests** (9 B1 + 13 B2 +
  13 B3 + 12 B4a).

### Task 4: Final CI gate + commit

- [ ] `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets
  --locked -- -D warnings`; `cargo build --workspace --locked`;
  `cargo test --workspace --locked` (total 3 core + 47 model + 120 parser).
- [ ] `git status --short`: only `arguments.rs`, `lib.rs`, and the two new
  `docs/snowball` files plus the hook-appended `observations.jsonl`.
- [ ] Commit B4a to `main` (source + docs), then commit hook observations if
  the diff is non-empty.

---

## Summary

B4a scaffolds the `arguments` module, implements `build_arguments` via TDD as a
mirror of B3's `build_statements`, adds 12 coverage tests, and closes with the
B1–B3 CI gate. One function, one new module, no new dependency; `argdown-mcp`
untouched. PCS roles / inference / argument↔PCS linkage are deferred to B4b.
