---
title: What should I do with the stray rust_out binary (untracked 473K executable, not part of any commit)
status: accepted
date: '2026-06-05T16:40:57.608Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_01JmB8Jt9xeZNj52soLYAnur
  supersedes: null
  tags:
    - ambient
---

# What should I do with the stray rust_out binary (untracked 473K executable, not part of any commit)

## Context and Problem Statement

Question category: rust_out.

## Considered Options

- **Delete it** — Remove the stray rust_out file from the repo root. It's a rustc default-output artifact left by a review step — not source, not committed. (Recommended.)
- **Leave it** — Keep the file in place. It stays untracked and won't be pushed, but remains in your working tree.

## Decision Outcome

Chose **Delete it**. Remove the stray rust_out file from the repo root. It's a rustc default-output artifact left by a review step — not source, not committed. (Recommended.)
