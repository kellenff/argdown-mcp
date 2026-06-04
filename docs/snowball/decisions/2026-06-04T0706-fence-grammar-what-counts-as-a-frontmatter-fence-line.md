---
title: Fence grammar — what counts as a frontmatter fence line
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

# Fence grammar — what counts as a frontmatter fence line

## Context and Problem Statement

Question category: Fence.

## Considered Options

- **Lenient mirror (3+ =, indent OK)** — A fence line = optional leading whitespace + `===` or more `=` + optional trailing whitespace, to end of line. Opening and closing counts need not match. Faithful to @argdown/core, which accepts both `====` and indented fences. Recommended.
- **Tight: exactly === at col 0** — Only an unindented line that is exactly `===` opens/closes frontmatter. Simpler regex, but rejects inputs argdown accepts.

## Decision Outcome

Chose **Lenient mirror (3+ =, indent OK)**. A fence line = optional leading whitespace + `===` or more `=` + optional trailing whitespace, to end of line. Opening and closing counts need not match. Faithful to @argdown/core, which accepts both `====` and indented fences. Recommended.
