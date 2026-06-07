# Benchmark Layer B Implementation Plan

**Goal:** A `crates/argdown-model/benches/model.rs` Criterion bench covering the
public Layer-B functions, mirroring the parser benches.

**Spec:** `docs/snowball/specs/2026-06-06-benchmark-layer-b-design.md`. **Branch:**
commit directly to `main` (purely additive — a new bench + dev-dep; no
production change, no version bump).

---

### Task 1: Cargo wiring
- [ ] `crates/argdown-model/Cargo.toml`: add `criterion = { workspace = true }`
  to `[dev-dependencies]`; add `[[bench]] name = "model", harness = false`.

### Task 2: Write model.rs bench
- [ ] Corpus generators (as code): `corpus(units)` (mixed) and `dialectic(args)`
  (attack chain); untimed guards (mixed parses; dialectic AF has > 0 attacks).
- [ ] `functions` group — one micro per public fn (`build_sections`,
  `build_statements`, `build_arguments`, `build_model`, `build_tags`,
  `parse_metadata`, `dung_framework`, `grounded_extension`); inputs built once
  outside the timing loop; `black_box` the arg.
- [ ] `scaling` group — `build_model` over mixed corpus 1/50/500, `Throughput::Bytes`.
- [ ] `dung_scaling` group — `dung_framework` + `grounded_extension` over
  `dialectic` at increasing sizes, `Throughput::Elements`.
- [ ] `criterion_group!` + `criterion_main!`; module doc with the run/baseline workflow.

### Task 3: Verify + commit
- [ ] `cargo bench -p argdown-model --bench model -- --test` (runs each bench once).
- [ ] `cargo clippy -p argdown-model --all-targets --locked -- -D warnings` (CI
  compiles benches) + `cargo fmt --all --check`; root `cargo test --workspace` still green.
- [ ] Commit spec + plan + Cargo.toml + model.rs to `main`.

---

## Summary
Additive Layer-B Criterion bench (per-function micros + build_model byte-scaling
+ Dung element-scaling), corpus-as-code, same boundary/local-guard philosophy as
the parser benches.
