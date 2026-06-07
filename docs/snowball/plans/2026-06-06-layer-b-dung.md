# Layer B Dung AF + Grounded Extension (B6b) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: snowball:test-driven-development. Finish with the two-stage review gate (spec-compliance + code-quality) + a holistic pass.

**Goal:** A new `argdown_model::dung` module with `dung_framework(&Model) ->
ArgumentationFramework` (sharp argument→argument-Attack projection) and
`grounded_extension(&ArgumentationFramework) -> GroundedLabelling`
(characteristic-function least-fixpoint). Pure, total. Project end goal: "Dung
extensions in Rust."

**Architecture:** New module `crates/argdown-model/src/dung.rs`; standalone
projection over the Model (no new Model fields). No new dependency. **Spec:**
`docs/snowball/specs/2026-06-06-layer-b-dung-design.md`. **Branch:** commit
directly to `main` (additive, no version bump).

---

### Task 1: Types + stubs + lib wiring
- [ ] `dung.rs`: `ArgumentationFramework { arguments, attacks }`,
  `GroundedLabelling { in_, out, undec }`, stub `dung_framework`/`grounded_extension`.
- [ ] `lib.rs`: `mod dung;` + `pub use dung::{ArgumentationFramework, GroundedLabelling, dung_framework, grounded_extension};`.
- [ ] Gate clean; existing tests green.

### Task 2: dung_framework (TDD)
- [ ] RED→GREEN: argument-level attack `<a>: A` / `  - <b>` / `<b>: B` →
  `attacks == [(b, a)]`. Filter `model.edges` to (from=Argument, to=Argument,
  kind=Attack), dedup, source order; `arguments` = all `model.arguments` ids.
- [ ] Tests: empty; statement-level attack → 0; support/undercut excluded; dedup;
  anonymous arg present.

### Task 3: grounded_extension (TDD)
- [ ] RED→GREEN: least-fixpoint over directly-built AFs. `attackers_of` map; label
  IN (all attackers OUT) / OUT (any attacker IN); iterate to fixpoint; leftover UNDEC.
- [ ] Tests: unattacked→IN; a→b → IN/OUT; chain→{a,c IN,b OUT}; self-attack→UNDEC;
  2-cycle→UNDEC. Plus an end-to-end matching the `dung_extensions` probe ({b IN, a OUT}).

### Task 4: Final gate + two-stage review + commit
- [ ] Full CI gate (`fmt --check`, `clippy -D warnings`, build, `test --workspace`).
- [ ] Two-stage subagent review (spec-compliance, then code-quality) + holistic; fix loops.
- [ ] Commit B6b (spec + plan + code) to `main`. Layer B (B1–B6) complete.

---

## Summary
B6b: sharp AF projection + grounded-extension fixpoint, standalone over the Model,
total, reference-faithful (probe-confirmed). TDD + two-stage review. Completes Layer B.
