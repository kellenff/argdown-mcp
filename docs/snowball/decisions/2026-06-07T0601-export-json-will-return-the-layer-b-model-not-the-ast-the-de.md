---
title: >-
  export_json will return the Layer B Model (not the AST). The debate argued that's a misnomer worth fixing before
  launch. You'd fixed the 'reference trio' names earlier — keep export_json, or rename
status: accepted
date: '2026-06-07T06:01:12.605Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_013JjKAu9aTNSKW5eSdCmkv1
  supersedes: null
  tags:
    - ambient
---

# export_json will return the Layer B Model (not the AST). The debate argued that's a misnomer worth fixing before launch. You'd fixed the 'reference trio' names earlier — keep export_json, or rename

## Context and Problem Statement

Question category: Tool name.

## Considered Options

- **Keep export_json + clear description** — Literal parity with the reference trio. Mitigate the misnomer with an explicit description: 'Returns the resolved Layer B model (statements, arguments, PCS roles, edges, conflicts) as JSON — not the raw AST or source.' Zero divergence from your earlier choice.
- **Rename to export_model** — Name the content, not the serialization format. The debate's preferred fix — honest, one-token change, and you're already shedding reference compat. Diverges from the reference trio by one name.
- **Rename to get_resolved_model** — Most explicit / agent-legible. Signals 'resolved semantics' clearly. Furthest from the reference naming.

## Decision Outcome

Chose **Rename to export_model**. Name the content, not the serialization format. The debate's preferred fix — honest, one-token change, and you're already shedding reference compat. Diverges from the reference trio by one name.
