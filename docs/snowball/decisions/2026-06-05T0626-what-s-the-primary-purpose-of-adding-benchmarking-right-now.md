---
title: What's the primary purpose of adding benchmarking right now
status: accepted
date: '2026-06-05T06:26:48.605Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_016n7kEWf5aZoT1zN6C2cBXp
  supersedes: null
  tags:
    - ambient
---

# What's the primary purpose of adding benchmarking right now

## Context and Problem Statement

Question category: Purpose.

## Considered Options

- **Baseline + regression guard** — Measure parse() throughput now, and keep the suite around so future increments (Layer B, etc.) can detect slowdowns. Fits the project's 'deterministic gates' + 'bake in early' philosophy. Doesn't necessarily gate CI yet.
- **Baseline visibility only** — Just establish current performance numbers for local insight. No commitment to tracking over time or wiring into CI.
- **Optimize a hot path** — You suspect/know something is slow and want benchmarks to drive an optimization. (Which path?)
- **Compare vs @argdown/core** — Benchmark this Rust parser against the reference TypeScript implementation to quantify the rewrite's speedup.

## Decision Outcome

Chose **Baseline + regression guard**. Measure parse() throughput now, and keep the suite around so future increments (Layer B, etc.) can detect slowdowns. Fits the project's 'deterministic gates' + 'bake in early' philosophy. Doesn't necessarily gate CI yet.
