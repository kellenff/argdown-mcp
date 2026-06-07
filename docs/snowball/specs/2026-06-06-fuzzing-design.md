# Parser Fuzzing — Design

- **Date:** 2026-06-06
- **Status:** Approved
- **Scope:** A coverage-guided fuzzing harness for the parser and the full
  Layer-B pipeline, via **cargo-fuzz** (libFuzzer). Two targets driving
  `parse` and the model build; assert no panic + structural invariants. A seed
  corpus from the bench corpus-as-code, and a nightly CI smoke-run.

## Context

`parse(&str) -> Result<Document, Error>` is the project's public boundary and is
meant to be **total in the panic sense** — every input yields `Ok` or `Err`,
never a panic, hang, or OOM. Layer B (`build_model` / `build_tags` /
`dung_framework` / `grounded_extension`) is explicitly **total** (no `Result`):
its contract is that *any* `Document` the parser can produce is processed
without panicking. Those are exactly the contracts fuzzing verifies. The ADR
also records that span-correctness bugs (CRLF drift, char-vs-byte,
record-before-whitespace) are the recurring defect class — so an in-bounds span
invariant is a high-value fuzz oracle, not just no-panic.

### Decisions (operator-settled)

1. **cargo-fuzz** (libFuzzer) over `bolero` or proptest-only. The standard
   Rust coverage-guided fuzzer. It needs **nightly** to run, so the `fuzz/`
   crate is its **own workspace** (an empty `[workspace]` table) and is not a
   member of the root workspace — the stable build / `clippy --all-targets` /
   `test --workspace` gate is entirely unaffected. Fuzzing is treated as a
   nightly/local dev tool (like a profiler), consistent with the project's
   stable-only stance for shipped crates and CI gates.
2. **Full-pipeline targets**, not parser-only: one target for `parse`, one that
   also runs Layer B on the parsed `Document`. Exercises the "Layer B is total"
   claim end-to-end.
3. **Oracles beyond no-panic** (libFuzzer already catches panics / hangs / OOM):
   parse determinism, an in-bounds span invariant over the whole AST, and AF /
   grounded-labelling well-formedness — turning silent-wrong-results into crashes
   the search can find.
4. **Seed corpus as code**, reusing the bench corpus constructs (one valid doc
   per grammar feature + a mixed doc), committed under `fuzz/seeds/`.
5. **CI smoke-run** (nightly) in a separate `fuzz.yml` workflow — build both
   targets and run each briefly on PR/push to `main` and on a daily schedule —
   to prevent bit-rot and catch regressions, without making coverage-guided
   fuzzing a blocking deterministic gate.

## Architecture

```
fuzz/
  Cargo.toml          # own workspace; libfuzzer-sys + the three crates
  .gitignore          # target/ corpus/ artifacts/ coverage/ (seeds/ committed)
  fuzz_targets/
    parse.rs          # parse(): no-panic + determinism + span-in-bounds
    model.rs          # parse() then Layer B: no-panic + AF/labelling invariants
  seeds/              # committed seed inputs (valid Argdown, per construct)
```

`fuzz/Cargo.toml` ends with an empty `[workspace]` so the root workspace
(`members = ["crates/*"]`) never sees it. Targets are `[[bin]]` with `test =
false, doc = false, bench = false`.

### `parse` target

```rust
fuzz_target!(|src: &str| {
    let Ok(doc) = parse(src) else { return };
    assert!(parse(src).as_ref() == Ok(&doc));   // determinism
    check_document_spans(&doc, src.len());        // every span ⊆ [0, len], start ≤ end
});
```

`check_document_spans` walks every span-bearing node — `Document.frontmatter`,
each `Block` (`Heading`/`Statement`/`Argument`/`Relation`/`Pcs`), nested
`PcsItem`s, relation targets, inline elements, and every `Metadata` block (on
statements, arguments, headings, inferences, frontmatter) — asserting
`start ≤ end ≤ src.len()` for each.

### `model` target

```rust
fuzz_target!(|src: &str| {
    let Ok(doc) = parse(src) else { return };
    let model = build_model(&doc);
    assert!(build_model(&doc) == model);         // determinism
    let _ = build_tags(&doc);
    let af = dung_framework(&model);
    let labelling = grounded_extension(&af);
    // every attack endpoint is a listed argument
    for &(from, to) in &af.attacks { assert!(af.arguments.contains(&from) && af.arguments.contains(&to)); }
    // the grounded labelling partitions the arguments exactly once
    assert_eq!(labelling.in_.len() + labelling.out.len() + labelling.undec.len(), af.arguments.len());
});
```

## Running

Locally (nightly + cargo-fuzz). libFuzzer writes new finds into the *first*
corpus directory, so never point a run at `fuzz/seeds/` directly — copy the
committed seeds into the gitignored managed corpus and fuzz that:

```
cargo +nightly fuzz build                              # just compile the targets

mkdir -p fuzz/corpus/parse && cp fuzz/seeds/*.argdown fuzz/corpus/parse/
cargo +nightly fuzz run parse                          # writes finds to fuzz/corpus/parse
```

The managed corpus (`fuzz/corpus/<target>`) and crash artifacts
(`fuzz/artifacts/`) are gitignored; the committed `fuzz/seeds/*.argdown`
bootstraps a run. (`fuzz/.gitignore` also pins `seeds/` to `*.argdown` so a
stray run can't commit generated inputs.)

## CI

`.github/workflows/fuzz.yml`: ubuntu + nightly + `cargo-fuzz`, on
`pull_request`/`push` to `main`, a daily `schedule`, and `workflow_dispatch`.
For each target: seed the corpus (`cp fuzz/seeds/*.argdown
fuzz/corpus/<t>/`) then `cargo +nightly fuzz run <t> -- -max_total_time=30`
(time-boxed). A discovered crash fails the job and uploads the artifact. This
stays out of the stable `ci.yml`.

## Out of scope

- Making coverage-guided fuzzing a blocking PR gate (non-deterministic search).
- A serializer → no parse→print→parse roundtrip oracle (none exists).
- Structure-aware (`Arbitrary`-derived) input generation — raw `&str` mutation
  with the seed corpus is enough to start; revisit if coverage plateaus.

## Summary

cargo-fuzz harness in an out-of-workspace `fuzz/` crate: two targets fuzzing
`parse` and the full Layer-B pipeline for no-panic plus determinism, span-in-
bounds, and AF well-formedness; seeded from the bench corpus; smoke-run nightly
in CI. The stable workspace and its gates are untouched.
