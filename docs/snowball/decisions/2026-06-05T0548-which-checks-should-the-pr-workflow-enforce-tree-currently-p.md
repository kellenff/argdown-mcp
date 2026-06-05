---
title: Which checks should the PR workflow enforce? (Tree currently passes all of these.)
status: accepted
date: '2026-06-05T05:48:30.420Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: c6d6d1c0-4521-4566-81c9-1253590e1bb5
  source_event_id: toolu_01YFeevwpENkk7KCqeo6PEE4
  supersedes: null
  tags:
    - ambient
---

# Which checks should the PR workflow enforce? (Tree currently passes all of these.)

## Context and Problem Statement

Question category: Checks.

## Considered Options

- **cargo test** — cargo test --workspace --locked. The core correctness gate.
- **cargo fmt --check** — Fails if code isn't rustfmt-canonical. Matches your 'automatic formatters, one canonical style' preference.
- **cargo clippy -D warnings** — Lint gate; any clippy warning fails CI. You already keep the tree clippy-clean (e.g. the import-order commit).
- **cargo build** — Plain debug build. Largely redundant if test runs (test compiles everything), but explicit.

## Decision Outcome

Chose **cargo test, cargo fmt --check, cargo clippy -D warnings, cargo build**. 
