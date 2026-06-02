---
title: >-
  How should we decompose the workspace into crates initially? (The existing root binary `argdown-mcp` becomes a
  workspace member either way.)
status: accepted
date: '2026-06-02T17:51:52.830Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01EjoKqFu7omtV8sp4dU69Aj
  supersedes: null
  tags:
    - ambient
---

# How should we decompose the workspace into crates initially? (The existing root binary `argdown-mcp` becomes a workspace member either way.)

## Context and Problem Statement

Question category: Crate layout.

## Considered Options

- **Two crates (parser + mcp)** — crates/argdown-parser (winnow-based lib, AST types live here) + crates/argdown-mcp (the MCP server binary depending on the parser). YAGNI — split out shared types only when a second consumer appears.
- **Three crates (core + parser + mcp)** — crates/argdown-core (shared AST/model + error types), crates/argdown-parser (winnow parser producing core types), crates/argdown-mcp (server). More separation upfront; useful if you foresee a CLI or other consumers soon.
- **Parser only for now** — Scaffold crates/argdown-parser and the virtual workspace; defer/remove the mcp binary until the parser stabilizes. Smallest first step.

## Decision Outcome

Chose **Three crates (core + parser + mcp)**. crates/argdown-core (shared AST/model + error types), crates/argdown-parser (winnow parser producing core types), crates/argdown-mcp (server). More separation upfront; useful if you foresee a CLI or other consumers soon.
