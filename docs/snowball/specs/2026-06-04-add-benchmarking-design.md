# Parser Benchmarks — Design

- **Date:** 2026-06-04
- **Status:** Approved
- **Scope:** Add a Criterion benchmark suite over the parser's public
  `parse()` entry point, to establish a performance baseline and serve as a
  local regression guard as the grammar grows. Local-only (`cargo bench`); no
  CI gating. Purely additive — no changes to existing source or the public API.

## Context

The parser has grown through increments A1–A5b with no performance
measurement. `argdown_parser::parse(&str) -> Result<Document, Error>` is the
single public entry point: a pure function, no I/O, deterministic, stable
across increments — the ideal benchmark target. With the grammar growing
additively (each increment adds variants/fields), a baseline now lets future
increments (Layer B, etc.) detect slowdowns. The project already gates on
deterministic tools (`cargo test`, `cargo clippy -D warnings`, `cargo fmt`);
benchmarks add a *temporal* guard (diff against a saved baseline) rather than a
committed golden number.

## Decisions

Settled with the operator (`ask-user-question`):

1. **Purpose — baseline + regression guard.** Measure `parse()` throughput now
   and keep the suite so later work can detect slowdowns. Not a hard CI gate.

2. **Run location — local-only (`cargo bench`).** No CI changes and no
   shared-runner timing noise. Criterion saves a baseline to `target/criterion`
   and prints `% change` vs the last run; that change-detection *is* the guard.
   CI tracking/gating can be layered on later if wanted.

3. **Framework — Criterion 0.5.** The de-facto standard; statistical (outlier
   detection, confidence intervals) and the only option with a turnkey
   cross-run baseline-comparison workflow. Nightly `#[bench]` was rejected (this
   project is stable-only, edition 2024); Divan was rejected for a weaker
   cross-run baseline story.

4. **Corpus — feature micros + size scaling, corpus-as-code.** One small input
   per grammar construct (so a regression names the exact recognizer) plus a
   representative mixed document at three sizes (end-to-end throughput +
   scaling). Inputs are generated/declared in code, not committed as `.argdown`
   blobs, so sizes are reproducible and version-controlled. Real-world-corpus
   was rejected for sourcing/licensing overhead and uncontrolled sizes.

5. **Layout — Approach A: single bench file in the owning crate.** One
   `crates/argdown-parser/benches/parse.rs` holding corpus + both bench groups.
   A shared/testable corpus module or dedicated bench crate (Approach C) was
   rejected as more structure than one bench file warrants; on-disk fixtures
   (Approach B) were rejected for committed blobs and awkward scaling.

## Layout

```
crates/argdown-parser/
  benches/
    parse.rs        # corpus (consts + generator) + two bench groups
  Cargo.toml        # + [dev-dependencies] criterion, + [[bench]]
```

Version centralized the way `winnow` is (one canonical place):

```toml
# root Cargo.toml → [workspace.dependencies]
criterion = "0.5"          # default features only — no html_reports/plotters

# crates/argdown-parser/Cargo.toml
[dev-dependencies]
criterion = { workspace = true }

[[bench]]
name = "parse"
harness = false
```

`html_reports` is deliberately **off**: the regression guard is the terminal
`% change`, not the SVG plots, and since CI compiles benches (`--all-targets`),
omitting `plotters` keeps that cost down. A one-line comment in `parse.rs` notes
how to enable it locally.

## Corpus (corpus-as-code)

**Feature micros** — one `const &str` per construct, each a small valid
document for: heading, statement, relation (a reference with an indented
relation cluster), PCS (premises + inference + conclusion), inline (italic /
bold / link / `@[]` / `@<>` / `#tag`), element metadata (`{…}`), and document
frontmatter (`===` fences).

**Scaling** — a generator `fn corpus(units: usize) -> String` emits frontmatter
once, then one representative mixed "unit" (heading + claim with inline +
metadata + a relation pair + a PCS) repeated `units` times. Benched at:

| name   | units | approx size |
| ------ | ----- | ----------- |
| small  | 1     | ~0.3 KB     |
| medium | 50    | ~12 KB      |
| large  | 500   | ~125 KB     |

The exact `corpus()` unit content is a domain judgment ("what is a
*representative* Argdown document?") to be refined with the operator at
implementation; the sizes above are starting points, tuned if the byte spread
proves uninformative.

## Benchmark structure

Two `benchmark_group`s, both timing only `parse()`:

- **`features`** — `bench_function` per construct, `Throughput::Bytes(src.len())`.
- **`scaling`** — `bench_with_input` over `[small, medium, large]` keyed by
  `BenchmarkId::from_parameter`, `Throughput::Bytes` per size so the series
  reads as MB/s.

Inputs pass through `std::hint::black_box`; the closure returns the `Result` so
the optimizer can't elide the parse.

**Corpus-validity guard.** Before timing each input,
`assert!(parse(src).is_ok(), …)` (untimed, once per bench). If a corpus string
were malformed, `parse()` would fail-fast early and the bench would measure the
*error path*, not parsing — the assert makes that panic loudly. Because
`harness = false` excludes the bench binary from `cargo test`'s `#[test]`
discovery (Approach A keeps the corpus in the bench), this in-bench assert *is*
the corpus correctness check.

## Regression-guard workflow (local)

```bash
cargo bench -p argdown-parser                          # auto-diffs vs last run → "change: +4.2% (p=0.00)"
cargo bench -p argdown-parser -- --save-baseline main  # snapshot before a change
# …make parser changes…
cargo bench -p argdown-parser -- --baseline main       # compare against the snapshot
```

Baselines live in `target/criterion/` — already covered by the `/target`
gitignore, so **no `.gitignore` change** and no committed golden numbers.

## CI impact & verification

- **`Cargo.lock` must be committed.** CI runs everything `--locked`; adding
  Criterion changes the lock, and `--locked` fails on a stale lock.
- The existing `cargo clippy --workspace --all-targets --locked -- -D warnings`
  will now **compile and lint `parse.rs` on all four platforms**, so the bench
  must be clippy-clean and cross-platform (Criterion is). Mirror the gate
  locally with `cargo clippy --all-targets -- -D warnings` and
  `cargo test --benches` (compile-smoke) before merge.
- No workflow file changes: benchmarks are not added to any CI job; they only
  enter the build graph via the existing `--all-targets` clippy step.

## Out of scope (YAGNI)

- CI gating, `github-action-benchmark`/gh-pages tracking.
- `iai`/instruction-count benchmarking.
- Comparison against the reference `@argdown/core`.
- Benchmarking private sub-recognizers (public `parse()` only, so the suite
  stays stable across internal refactors).
- Committed baselines / golden numbers.
