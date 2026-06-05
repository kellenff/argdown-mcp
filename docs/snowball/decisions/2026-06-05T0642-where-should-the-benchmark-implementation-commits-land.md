---
title: Where should the benchmark implementation commits land
status: accepted
date: '2026-06-05T06:42:40.523Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_01MJu7DXPCcpMWpvNdd1vRVm
  supersedes: null
  tags:
    - ambient
---

# Where should the benchmark implementation commits land

## Context and Problem Statement

Question category: Branch.

## Considered Options

- **Directly on main** — Matches your standing, ADR-documented convention (re-confirmed through A5b). Safe here because this change doesn't bump the version, so the version-gated release workflow won't fire. (Recommended — consistent with prior increments.)
- **Feature branch** — Create e.g. feat/add-benchmarking, commit there, open a PR. The CI gate runs on the PR; you merge when ready. Extra ceremony vs. your usual flow, but keeps main untouched until review.
- **Git worktree** — Isolated workspace via snowball:using-git-worktrees, then branch + PR. Most isolation; heaviest setup. Useful only if you want to keep this workspace free for other work.

## Decision Outcome

Chose **Directly on main**. Matches your standing, ADR-documented convention (re-confirmed through A5b). Safe here because this change doesn't bump the version, so the version-gated release workflow won't fire. (Recommended — consistent with prior increments.)
