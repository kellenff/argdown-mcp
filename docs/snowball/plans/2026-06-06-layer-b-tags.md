# Layer B Tags (B6a) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: snowball:test-driven-development.

**Goal:** A new `argdown_model::tags` module whose `build_tags(&Document) ->
Tags` collects inline `#tag`s into a first-occurrence-ordered registry plus a
per-block (`Vec<Vec<TagId>>`) assignment.

**Architecture:** New module `crates/argdown-model/src/tags.rs`; no new
dependency. Pure and total. **Spec:**
`docs/snowball/specs/2026-06-06-layer-b-tags-design.md`. **Branch:** commit
directly to `main` (additive, no version bump).

---

### Task 1: Types + stub + lib wiring
- [ ] Create `tags.rs` with `TagId`, `Tags { tags, block_tags }`, and a stub
  `build_tags(_document) -> Tags::default()`.
- [ ] `lib.rs`: `mod tags;` + `pub use tags::{Tags, TagId, build_tags};`.
- [ ] Gate (`fmt`/`clippy -D warnings`/`build`); existing tests stay green.

### Task 2: Implement build_tags (TDD)
- [ ] RED→GREEN: happy path (`[A]: a #foo #bar` → `tags == ["foo","bar"]`,
  `block_tags == [[0,1]]`).
- [ ] Single pass over blocks; collect `InlineKind::Tag` from statement/argument
  bodies at every site (top-level statement/argument, PCS numbered statements,
  relation targets); register first-occurrence ids in a `HashMap<String,TagId>`;
  per-block deduped `Vec<TagId>`.

### Task 3: Coverage tests
- [ ] The 9 spec tests (empty, statement tags, argument tags, cross-block
  ordering, in-block dedup, PCS-statement tag, relation-target tag, no-tags,
  index alignment).
- [ ] Gate: `cargo test -p argdown-model`.

### Task 4: Final gate + review + commit
- [ ] Full CI gate (`fmt --check`, `clippy -D warnings`, build, `test --workspace`).
- [ ] A code-review pass (mechanical slice — single reviewer suffices); fix findings.
- [ ] Commit B6a (spec + plan + code) to `main`.

---

## Summary
B6a: `build_tags` over `&Document` → tag registry + per-block assignment, total,
additive. TDD + a review pass. Metadata promotion and per-entity aggregation
deferred; the Dung map is B6b.
