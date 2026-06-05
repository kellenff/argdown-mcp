---
title: Where should the benchmarks run — local-only, or wired into the existing CI
status: accepted
date: '2026-06-05T06:27:27.828Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_01Btg5MWDgsMNBB3o5w6TpHQ
  supersedes: null
  tags:
    - ambient
---

# Where should the benchmarks run — local-only, or wired into the existing CI

## Context and Problem Statement

Question category: Run location.

## Considered Options

- **Local-only (cargo bench)** — Run on demand locally; the framework saves baselines so you compare runs by hand before merging. No CI changes, no shared-runner noise. Simplest start; CI can be layered on later. (Recommended given 'doesn't necessarily gate CI yet'.)
- **CI tracking, non-gating** — Run benches in CI and record results over time (e.g. github-action-benchmark on gh-pages), alert on regression but never fail the build. Visibility across commits without false-failure risk on noisy runners.
- **CI gating (hard fail)** — A regression past a threshold fails the PR. Strongest guard, but wall-clock timing on shared runners is noisy — this realistically pushes toward instruction-count benchmarking (iai-callgrind + valgrind) for determinism.

## Decision Outcome

Chose **Local-only (cargo bench)**. Run on demand locally; the framework saves baselines so you compare runs by hand before merging. No CI changes, no shared-runner noise. Simplest start; CI can be layered on later. (Recommended given 'doesn't necessarily gate CI yet'.)
