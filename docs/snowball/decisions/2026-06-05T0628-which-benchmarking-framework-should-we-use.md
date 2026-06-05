---
title: Which benchmarking framework should we use
status: accepted
date: '2026-06-05T06:28:13.436Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_011exrKZ23YBfmma6HHh2rdL
  supersedes: null
  tags:
    - ambient
---

# Which benchmarking framework should we use

## Context and Problem Statement

Question category: Framework.

## Considered Options

- **Criterion** — The de-facto Rust standard. Auto-saves a baseline to target/criterion and prints % change vs the previous run on every invocation — that change-detection IS your regression guard. Statistical (outlier detection, confidence intervals). Cost: a dev-dependency and slightly slower bench compiles.
- **Divan** — Newer, ergonomic attribute-macro API, fast compiles, clean output. Lighter weight. Downside: cross-run baseline comparison is less turnkey than Criterion's, so the regression-guard workflow is more manual.
- **Built-in #[bench]** — std test::Bencher with no extra deps — but it's nightly-only. This project is on stable (edition 2024, CI on stable toolchain), so this would fracture the toolchain story. Not recommended.

## Decision Outcome

Chose **Criterion**. The de-facto Rust standard. Auto-saves a baseline to target/criterion and prints % change vs the previous run on every invocation — that change-detection IS your regression guard. Statistical (outlier detection, confidence intervals). Cost: a dev-dependency and slightly slower bench compiles.
