---
title: Which fuzzing framework
status: accepted
date: '2026-06-07T01:18:12.743Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01NFkbW9sRHdGZVok5bUo1Vq
  supersedes: null
  tags:
    - ambient
---

# Which fuzzing framework

## Context and Problem Statement

Question category: Framework.

## Considered Options

- **cargo-fuzz** — The standard Rust coverage-guided fuzzer (libFuzzer). Needs nightly to RUN; the fuzz/ crate is excluded from the workspace so stable build/CI/test stay untouched. Most powerful; fuzzing is treated as a nightly/local dev tool (like a profiler).
- **bolero** — Write each target once; it runs as a stable `cargo test` (corpus replay + bounded random) AND under nightly for coverage-guided fuzzing. Bridges the stable-only CI (targets become regular tests), at the cost of an extra harness dependency.
- **proptest only** — Stable generative property tests with `arbitrary`-style inputs — no coverage guidance. Fully stable, simplest, but not true (coverage-guided) fuzzing.

## Decision Outcome

Chose **cargo-fuzz**. The standard Rust coverage-guided fuzzer (libFuzzer). Needs nightly to RUN; the fuzz/ crate is excluded from the workspace so stable build/CI/test stay untouched. Most powerful; fuzzing is treated as a nightly/local dev tool (like a profiler).
