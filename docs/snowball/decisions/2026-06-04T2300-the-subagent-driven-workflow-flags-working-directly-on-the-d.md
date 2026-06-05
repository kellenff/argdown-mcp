---
title: >-
  The subagent-driven workflow flags working directly on the default branch. A1–A5a were all committed straight to
  `main` (a deliberate earlier decision). Where should the A5b implementation commits go
status: accepted
date: '2026-06-04T23:00:16.310Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: bd91c155-11f1-49bc-9671-18dae30b53ed
  source_event_id: toolu_0171X4Bas84QrukRjNFBcaZd
  supersedes: null
  tags:
    - ambient
---

# The subagent-driven workflow flags working directly on the default branch. A1–A5a were all committed straight to `main` (a deliberate earlier decision). Where should the A5b implementation commits go

## Context and Problem Statement

Question category: Branch policy.

## Considered Options

- **Stay on main** — Commit implementation directly to main, consistent with how A1–A5a were done. No branch/worktree overhead.
- **New feature branch** — Create feat/a5b-frontmatter off main and commit there; you can merge/PR when done via finishing-a-development-branch.

## Decision Outcome

Chose **Stay on main**. Commit implementation directly to main, consistent with how A1–A5a were done. No branch/worktree overhead.
