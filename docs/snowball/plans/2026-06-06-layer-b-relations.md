# Layer B Relations (B5) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: snowball:test-driven-development. Finish with the two-stage review gate (spec-compliance + code-quality) + a holistic pass.

**Goal:** Extend `argdown_model::model::build_model` to resolve the flat
`Relation` AST into deduped directed `Edge`s between `Node`s, minting any
relation-participant nodes the registry lacks, and adding `edges` +
`relation_issues` to `Model`.

**Architecture:** Add `Node`, `RelationKind`, `Edge`, `RelationIssue` to
`crates/argdown-model/src/model.rs`; add `edges`/`relation_issues` to `Model`;
fold relation resolution into the existing `build_model` document walk via a
monotonic indent stack (peek-then-enter). No new external dependency. Pure and
total. Additive over B3/B4a/B4b.

**Spec:** `docs/snowball/specs/2026-06-06-layer-b-relations-design.md`

**Branch:** Commit directly to `main` — additive (no version bump).

---

### Task 1: Types + Model fields (scaffold, gate clean)
- [ ] Add `Node`, `RelationKind`, `Edge`, `RelationIssue` types; add
  `edges: Vec<Edge>` and `relation_issues: Vec<RelationIssue>` to `Model`
  (both `Default`-empty so existing B4b tests still pass).
- [ ] `lib.rs`: re-export the new public types.
- [ ] Gate (`fmt`/`clippy`/`build`/`test`) — all existing 70 model tests stay green.

### Task 2: Top-level relation resolution (TDD)
- [ ] RED→GREEN: extend `build_model`'s block walk with a `Frame` stack +
  `enter` helper; `Block::Statement`/`Argument` enter at indent 0 (mint
  singleton for plain statements); `Block::Relation` peek-then-enter:
  resolve parent, resolve target (merge/mint node), `orient` by direction,
  `push_if_new` (dedup by from/to/kind), enter target.
- [ ] `RelationWithoutParent` for parentless relations.
- [ ] Tests: direction (support/attack/outbound/contradictory), nesting/source,
  plain & argument targets, dedup, parentless. Verify against reference probes.

### Task 3: PCS-interspersed relations (TDD)
- [ ] Resolve `PcsItem::Relation` per-PCS (local scope; source = preceding PCS
  statement / deeper relation target), feeding the same `edges` set + dedup.
- [ ] Tests: `(1) [P]\n  +> [S]\n----\n(2) C` ⇒ edge P→S; B4b roles unaffected.

### Task 4: Completeness + final gate + review
- [ ] Tests: relation-target statements present in `Model.statements`;
  prefix-correspondence with B3/B4a/B4b preserved.
- [ ] Full CI gate (`fmt --check`, `clippy -D warnings`, `build`, `test --workspace`).
- [ ] Two-stage subagent review (spec-compliance, then code-quality) + holistic; fix loops; commit to `main`.

---

## Summary
B5 folds relation resolution into `build_model`: deduped directed edges between
nodes, complete node registry, monotonic indent stack. Reference-faithful,
total, additive. TDD + two-stage review.
