---
title: How should I land these commits? (Pushing to main triggers the v0.1.0 release immediately.)
status: accepted
date: '2026-06-05T05:44:01.590Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: c6d6d1c0-4521-4566-81c9-1253590e1bb5
  source_event_id: toolu_01BLsCMpXkk67tjp3QF7aZ1Z
  supersedes: null
  tags:
    - ambient
---

# How should I land these commits? (Pushing to main triggers the v0.1.0 release immediately.)

## Context and Problem Statement

Question category: Land how.

## Considered Options

- **Branch + PR** — Create a branch, commit both files, push, open a PR. Nothing releases until you merge. Safe review step; you control when v0.1.0 actually publishes. (Recommended — you're currently on main.)
- **Commit to main, don't push** — Commit both files directly on main locally but do NOT push. No release fires until you push yourself later.
- **Commit + push to main** — Commit to main and push now. The workflow lands and immediately cuts a public v0.1.0 GitHub Release of the placeholder binary.

## Decision Outcome

Chose **Branch + PR**. Create a branch, commit both files, push, open a PR. Nothing releases until you merge. Safe review step; you control when v0.1.0 actually publishes. (Recommended — you're currently on main.)
