---
title: Opening fence at doc start with NO closing `===` before EOF
status: accepted
date: '2026-06-04T07:06:17.997Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: bd91c155-11f1-49bc-9671-18dae30b53ed
  source_event_id: toolu_01QezNSrYnUcvTTEiwWFY91B
  supersedes: null
  tags:
    - ambient
---

# Opening fence at doc start with NO closing `===` before EOF

## Context and Problem Statement

Question category: Unterminated.

## Considered Options

- **Hard error (fail-fast)** — Unterminated frontmatter is an `Err{message, offset}`, consistent with A5a's unterminated-`{` rule and the parser's strict fail-fast model. Recommended.
- **Treat opener as statement text** — If no closing fence is found, back off: the `===` is just an ordinary statement, no frontmatter. More lenient but ambiguous about intent.

## Decision Outcome

Chose **Hard error (fail-fast)**. Unterminated frontmatter is an `Err{message, offset}`, consistent with A5a's unterminated-`{` rule and the parser's strict fail-fast model. Recommended.
