---
title: >-
  Implementation will run on the current branch, which is `main`. How do you want to isolate the work? (Your decision
  logs show prior increments were done directly on main, so I'm asking rather than assuming.)
status: accepted
date: '2026-06-07T16:32:15.793Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01KqzxKWkGgjHcPpTnvhQoTY
  supersedes: null
  tags:
    - ambient
---

# Implementation will run on the current branch, which is `main`. How do you want to isolate the work? (Your decision logs show prior increments were done directly on main, so I'm asking rather than assuming.)

## Context and Problem Statement

Question category: Branch.

## Considered Options

- **Feature branch** — Create a branch like feat/argdown-cli off main and run all task commits there. Keeps main clean; merge/PR when done via finishing-a-development-branch. Recommended for a multi-crate change.
- **Work on main** — Commit directly to main, matching this project's established pattern. Fastest, no branch ceremony, but no isolation if something needs unwinding.
- **Git worktree** — Spin up an isolated worktree (snowball:using-git-worktrees) so the work lives in a separate checkout entirely. Heaviest isolation.

## Decision Outcome

Chose **Work on main**. Commit directly to main, matching this project's established pattern. Fastest, no branch ceremony, but no isolation if something needs unwinding.
