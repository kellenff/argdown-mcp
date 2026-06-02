---
title: We're on the default branch `main` (only the two doc commits so far). Where should I implement the workspace
status: accepted
date: '2026-06-02T18:06:10.629Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01WPe3Ci3AEocSSKnA5hFt7b
  supersedes: null
  tags:
    - ambient
---

# We're on the default branch `main` (only the two doc commits so far). Where should I implement the workspace

## Context and Problem Statement

Question category: Branch.

## Considered Options

- **On main** — Commit the four tasks directly to main. Natural choice for initializing a fresh project where main only has setup commits. (Recommended)
- **New branch** — Create a feature branch (e.g. feat/cargo-workspace) and commit there, leaving main untouched until you merge.
- **Git worktree** — Create an isolated git worktree to work in. Heavier; usually for parallel work, not single-developer init.

## Decision Outcome

Chose **On main**. Commit the four tasks directly to main. Natural choice for initializing a fresh project where main only has setup commits. (Recommended)
