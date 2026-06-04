---
title: A `===` fence line that appears AFTER real content (not at doc start) — how to treat it
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

# A `===` fence line that appears AFTER real content (not at doc start) — how to treat it

## Context and Problem Statement

Question category: Non-leading ===.

## Considered Options

- **Plain statement text (thin)** — Frontmatter is recognized ONLY at document start. Elsewhere a `===` line falls through to normal parsing and becomes ordinary statement text. No new error path; keeps the increment thin. Diverges from argdown (which errors). Recommended for A5b scope.
- **Hard error (mirror argdown)** — A fence line anywhere but doc-start is a hard error, matching argdown's 'Invalid paragraph start'. Catches misplaced frontmatter, but adds a fence check at every block boundary.

## Decision Outcome

Chose **Hard error (mirror argdown)**. A fence line anywhere but doc-start is a hard error, matching argdown's 'Invalid paragraph start'. Catches misplaced frontmatter, but adds a fence check at every block boundary.
