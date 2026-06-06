# Layer B PCS + Model Aggregate (B4b) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: snowball:test-driven-development. Large slice — finish with the two-stage review gate (spec-compliance + code-quality) and a final holistic review.

**Goal:** A new `argdown_model::model` module whose `build_model(&Document) ->
Model` resolves PCS roles/inference, links each PCS to its argument (strict
adjacency, minting anonymous arguments for standalone PCSs), and produces the
first `Model` aggregate — the complete, unified statement and argument
registries (titled + untitled; named + anonymous).

**Architecture:** New module `crates/argdown-model/src/model.rs`. Reuses
`build_statements`/`build_arguments` internally + B2's `parse_metadata`. **No
new external dependency, no `Cargo.toml` change.** Pure and total. Additive over
B3/B4a (their source untouched).

**Spec:** `docs/snowball/specs/2026-06-06-layer-b-pcs-model-design.md`

**Branch:** Commit directly to `main` — additive (new module; no version bump).

---

## File Structure

| File | Change |
| ---- | ------ |
| `crates/argdown-model/src/model.rs` | **Create** — `PcsId`/`Role`/`ResolvedPcsItem`/`ResolvedPcs`/`ModelStatement`/`ModelArgument`/`PcsIssue`/`Model`, `build_model`, tests |
| `crates/argdown-model/src/lib.rs` | **Modify** — `mod model;` + one `pub use` |

---

### Task 1: Scaffold types + stub + wire lib.rs

- [ ] Create `model.rs` with all eight types (per spec) and a stub
  `pub fn build_model(_document: &Document) -> Model { Model::default() }`.
- [ ] `lib.rs`: add `mod model;` and
  `pub use model::{Model, ModelArgument, ModelStatement, PcsId, PcsIssue, ResolvedPcs, ResolvedPcsItem, Role, build_model};`.
- [ ] Gate: `cargo fmt`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo build --workspace --locked`. Stub builds clean.

### Task 2: Seed registries + linkage (TDD)

- [ ] RED→GREEN: `build_model` calls `build_statements`/`build_arguments`,
  seeds `Model.statements`/`arguments` (prefix-correspondence), builds
  title→id and title→first-def-span maps.
- [ ] Linkage: each `Block::Pcs` owned by the immediately-preceding
  `Block::Argument` (def or ref) via title map, else a minted anonymous
  `ModelArgument` (`title: None`); set owner's `pcs`; fill `block_pcs`.
- [ ] Tests: `<A>:`·PCS and bare `<A>`·PCS attach; `<A>`·section·PCS and
  `<A>`·statement·PCS detach (anonymous); two PCSs after one arg → 2nd
  anonymous; PCS with no preceding arg → anonymous.

### Task 3: Statement resolution + unified conflicts (TDD)

- [ ] For each `PcsItem::Statement`: titled → find-or-create by title (merge
  with top-level + other PCS occurrences; definition fills canonical or records
  `StatementConflict` using the first-def-span map); untitled → mint singleton
  `ModelStatement` (`title: None`, `canonical_text: Some(text)`).
- [ ] Tests: titled PCS statement merges with top-level same-title (one id);
  PCS-only titled gets its own class; untitled are distinct singletons; a
  top-level def redefined in a PCS records a conflict; `canonical_metadata`
  parsed via B2.

### Task 4: Role + inference resolution (TDD)

- [ ] Per PCS linear pass: bind inference→next-statement (`concludes_item_idx`),
  relations transparent; positional roles; finalize last-conclusion =
  `MainConclusion`, earlier = `IntermediaryConclusion`, unbound = `Premise`.
- [ ] Issues: `InferenceWithNoConclusion` (trailing/unbound), `ConsecutiveInferences`, `EmptyPcs`.
- [ ] Tests: single-step (premise/main); two-step (premise→intermediary→premise→main); premises-only; bare `----` vs `-- Rule, Rule --` rules captured; trailing inference; consecutive inferences; interspersed relation does NOT break binding.

### Task 5: Comprehensive suite + final gate + review

- [ ] Round out tests: `block_pcs` length == `document.blocks` length, non-PCS → None; prefix-correspondence (Model prefixes equal B3/B4a outputs); empty document → `Model::default()`.
- [ ] Final CI gate: `cargo fmt --all -- --check`; clippy `-D warnings`; build; `cargo test --workspace --locked` (model count = 47 + new B4b tests).
- [ ] Two-stage review (spec-compliance, then code-quality) via subagents; fix loops; final holistic review. Commit to `main`.

---

## Summary

B4b creates the `model` module: `build_model` resolves PCS structure and builds
the first complete `Model` aggregate, reusing B3/B4a and grounded in the
`@argdown/core` reference probes. One new module, no new dependency, additive.
Total function; conflicts and malformations are data.
