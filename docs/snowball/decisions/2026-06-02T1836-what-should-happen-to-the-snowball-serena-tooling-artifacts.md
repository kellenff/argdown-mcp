---
title: What should happen to the snowball/serena tooling artifacts
status: accepted
date: '2026-06-02T18:36:30.532Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01Pzp63XWNew3ApLNF7oUUZN
  supersedes: null
  tags:
    - ambient
---

# What should happen to the snowball/serena tooling artifacts

## Context and Problem Statement

Question category: Tooling files.

## Considered Options

- **Gitignore all of it** — Add .serena/ and docs/snowball/decisions/ to .gitignore. Keeps the repo to project code + specs/plans only.
- **Commit decisions, ignore the rest** — Commit the MADR .md decision records (design history, sits with the already-committed specs/plans), but gitignore .serena/ and observations.jsonl.
- **Commit everything** — Commit docs/snowball/decisions/ in full (incl. the 72K observations.jsonl). The snowball default — decision trail rides with the work.
- **Leave untracked** — Do nothing; the files stay untracked and out of git for now.

## Decision Outcome

Chose **Commit everything**. Commit docs/snowball/decisions/ in full (incl. the 72K observations.jsonl). The snowball default — decision trail rides with the work.
