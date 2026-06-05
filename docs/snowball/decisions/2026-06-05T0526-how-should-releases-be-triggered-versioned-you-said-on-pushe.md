---
title: How should releases be triggered/versioned? You said "on pushes to main" — literally every push, or gated
status: accepted
date: '2026-06-05T05:26:12.137Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: c6d6d1c0-4521-4566-81c9-1253590e1bb5
  source_event_id: toolu_018U8oTQNJjKaWAeS5s9t1u4
  supersedes: null
  tags:
    - ambient
---

# How should releases be triggered/versioned? You said "on pushes to main" — literally every push, or gated

## Context and Problem Statement

Question category: Trigger.

## Considered Options

- **On version bump only** — Workflow runs on every push to main but only releases when crates.io version in Cargo.toml changes. Clean, conventional, no release spam. Tag = the version (e.g. v0.1.1).
- **Every push to main** — A new release on literally every push, version auto-derived (e.g. 0.1.0+<short-sha> or a rolling 'latest' release that gets overwritten). High volume of releases.
- **Tag push** — Release only when you push a git tag like v0.1.0. Most explicit/manual control, but not 'on push to main' — you drive it by tagging.

## Decision Outcome

Chose **On version bump only**. Workflow runs on every push to main but only releases when crates.io version in Cargo.toml changes. Clean, conventional, no release spam. Tag = the version (e.g. v0.1.1).
