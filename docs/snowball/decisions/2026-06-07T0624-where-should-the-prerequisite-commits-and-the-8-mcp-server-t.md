---
title: Where should the prerequisite commits and the 8 MCP-server tasks land
status: accepted
date: '2026-06-07T06:24:59.239Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_01DaM4fbeMowipPZwTLnWEGW
  supersedes: null
  tags:
    - ambient
---

# Where should the prerequisite commits and the 8 MCP-server tasks land

## Context and Problem Statement

Question category: VC strategy.

## Considered Options

- **Stay on main** — Commit export/import + decision logs to main, then run the 8 tasks on main (each commits per the plan). Matches this repo's history — all prior work went straight to main, no feature branches.
- **New feature branch** — Create feat/argdown-mcp-server from main; commit export/import + logs there, then the tasks; finish via a PR/merge at the end. Isolates the work from main.
- **Git worktree** — Isolated worktree on a new branch (separate working dir). Same isolation as a branch; needs the export/import commit first so the worktree includes to_json.

## Decision Outcome

Chose **New feature branch**. Create feat/argdown-mcp-server from main; commit export/import + logs there, then the tasks; finish via a PR/merge at the end. Isolates the work from main.
