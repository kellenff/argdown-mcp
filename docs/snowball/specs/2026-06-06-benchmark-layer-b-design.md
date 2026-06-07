# Benchmark Layer B — Design

- **Date:** 2026-06-06
- **Status:** Approved
- **Scope:** Extend the Criterion benchmark suite to cover Layer B
  (`argdown-model`), mirroring the parser benchmarks. A new
  `crates/argdown-model/benches/model.rs` measures the public Layer-B functions
  over a corpus-as-code; no change to the parser benches.

## Context

The benchmark suite so far covers only `parse()` (in `argdown-parser`). Layer B
is now substantial (B1–B6) and deserves the same standing regression guard. The
established benchmarking decisions carry over unchanged: **Criterion 0.5**,
**measure at the public boundary** (never private helpers), **corpus-as-code**
(no committed fixture blobs), **local-only `cargo bench` regression guard** (the
gitignored `target/criterion` baseline + `% change`), and **one bench file in
the owning crate**. This is an expansion within that policy, not a new decision.

## What is benchmarked

The public Layer-B surface. Per the "one micro per construct so a regression
names the exact recognizer" principle, one micro per **function** so a
regression names the exact slice:

- `build_sections` (B1), `build_statements` (B3), `build_arguments` (B4a),
  `build_model` (the B4b aggregate, subsumes B3/B4a/B4b/B5), `build_tags` (B6a)
  — each over `&Document`.
- `parse_metadata` (B2) — over a `&Metadata` extracted from a metadata-rich
  statement.
- `dung_framework` (B6b) — over a `&Model`; `grounded_extension` (B6b) — over a
  `&ArgumentationFramework`.

Layer-B functions take `&Document` / `&Model` / `&AF`, so each bench parses /
builds its input **once, untimed** (the parse cost is the parser bench's
concern), and times only the Layer-B call.

## Corpus (as code)

Two generators, reproducible in code (mirroring the parser bench's `corpus`):

- **Mixed** `corpus(units)` — frontmatter once, then `units` representative
  blocks (heading; a claim with inline markup + metadata; an argument; a
  reference with a relation pair; a PCS). Drives the `&Document` builders.
- **Dialectic** `dialectic(args)` — `args` arguments in an **argument-level
  attack chain** (`<Arg i>\n  - <Arg i+1>`), so the AF is non-trivial. Drives
  `dung_framework` / `grounded_extension` (whose cost is attack-graph-dependent,
  not byte-dependent). A guard asserts the built AF has > 0 attacks, untimed.

## Groups

- **`functions`** — one micro per public Layer-B function over a fixed
  representative input (mixed `corpus(20)` for the `&Document`/`&Model` builders;
  a `dialectic` doc for the Dung pair; a metadata block for `parse_metadata`).
- **`scaling`** — `build_model` over the mixed corpus at small/medium/large
  (1 / 50 / 500 units), `Throughput::Bytes` (MB/s), mirroring the parser
  scaling bench — the aggregate is the representative end-to-end Layer-B cost.
- **`dung_scaling`** — `dung_framework` + `grounded_extension` over the dialectic
  corpus at increasing argument counts, `Throughput::Elements(args)`, surfacing
  the grounded-labelling fixpoint's scaling separately from byte size.

## Architecture

`crates/argdown-model/Cargo.toml`: add `criterion = { workspace = true }` to
`[dev-dependencies]` (alongside the existing `argdown-parser` dev-dep used to
build inputs) and a `[[bench]] name = "model", harness = false`. No production
dependency change; `argdown-mcp` untouched.

## Out of scope

- CI gating (benchmarks stay local-only, per the existing decision).
- Benching private sub-functions (`resolve_*`, the `Builder` internals) — only
  the public boundary, so the suite survives internal refactors (e.g. the B5
  `Builder` refactor).
- Throughput baselines committed as numbers (the gitignored Criterion baseline
  is the guard).

## Summary

A new `model.rs` Criterion bench covering every public Layer-B function, with
mixed + dialectic corpora as code, in three groups (per-function micros,
`build_model` byte-scaling, Dung element-scaling). Same framework, boundary, and
local-guard philosophy as the parser benches.
