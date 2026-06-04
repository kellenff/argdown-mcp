---
title: A5-full triggered the decomposition flag and is the most complex increment yet. Keep it as one increment, or split
status: accepted
date: '2026-06-04T02:34:34.743Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_01HesvMtuZpHtTuKQmCM7MJc
  supersedes: null
  tags:
    - ambient
---

# A5-full triggered the decomposition flag and is the most complex increment yet. Keep it as one increment, or split

## Context and Problem Statement

Question category: A5 split?.

## Considered Options

- **Split: A5a then A5b** — A5a = element {yaml} metadata (statement, argument, heading, inference) — the brace recognizer, multi-line, strict text-after, the B representation, reused at the 4 sites. A5b = document frontmatter (=== fences). Two independent features, each its own spec→plan→build; matches how A2 was split. Smaller, safer increments. (Recommended given the decomposition flag.)
- **Keep full A5** — All of it in one increment (element metadata + frontmatter), as you chose at the scope question. One cohesive metadata increment, but the largest/most-complex slice so far — against the thin-vertical-slice philosophy the decomposition flag is pointing at.

## Decision Outcome

Chose **Split: A5a then A5b**. A5a = element {yaml} metadata (statement, argument, heading, inference) — the brace recognizer, multi-line, strict text-after, the B representation, reused at the 4 sites. A5b = document frontmatter (=== fences). Two independent features, each its own spec→plan→build; matches how A2 was split. Smaller, safer increments. (Recommended given the decomposition flag.)
