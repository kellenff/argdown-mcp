---
title: What should the first parser increment (A) cover? This sets the boundary for this spec; later increments add the rest.
status: accepted
date: '2026-06-02T18:53:11.867Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_017TcwEsyGWsrV7ujMrwU6ar
  supersedes: null
  tags:
    - ambient
---

# What should the first parser increment (A) cover? This sets the boundary for this spec; later increments add the rest.

## Context and Problem Statement

Question category: Increment 1.

## Considered Options

- **Relational core (A1+A2)** — Sections/headings, statements (plain + titled + references), arguments (defs + refs), and nested relations (all symbols). Statement text kept as raw strings (no inline parsing yet). Defer PCS, inline, metadata. A coherent, genuinely-Argdown first slice. (Recommended)
- **Spine only (A1)** — Just sections/headings, statements (plain + titled), and comments. Smallest possible slice, but no arguments or relations — not yet recognizably Argdown.
- **Relational core + PCS (A1+A2+A3)** — Everything in relational core plus premise-conclusion structures and inference markers. More complete, but a larger first spec/plan.

## Decision Outcome

Chose **Spine only (A1)**. Just sections/headings, statements (plain + titled), and comments. Smallest possible slice, but no arguments or relations — not yet recognizably Argdown.
