---
title: Source position tracking in the AST
status: accepted
date: '2026-06-02T18:57:01.022Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_0116iDrZ3fHZigZTVqg5qiqi
  supersedes: null
  tags:
    - ambient
---

# Source position tracking in the AST

## Context and Problem Statement

Question category: Spans.

## Considered Options

- **Byte spans now** — Every node carries a Span { start, end } of byte offsets (via winnow's LocatingSlice). Line/column is computed at the output boundary when needed (e.g. JSON-compat, diagnostics). Retrofitting spans later is painful, so bake them in. (Recommended)
- **No spans yet** — Skip positions for A1, add them in a later increment. Less code now, but an invasive retrofit across every node and combinator later.

## Decision Outcome

Chose **Byte spans now**. Every node carries a Span { start, end } of byte offsets (via winnow's LocatingSlice). Line/column is computed at the output boundary when needed (e.g. JSON-compat, diagnostics). Retrofitting spans later is painful, so bake them in. (Recommended)
