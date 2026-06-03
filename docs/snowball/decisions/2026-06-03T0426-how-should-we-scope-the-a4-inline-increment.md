---
title: How should we scope the A4 (inline) increment
status: accepted
date: '2026-06-03T04:26:31.293Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_015WT7MwGLXEN5LYfCLjYqxp
  supersedes: null
  tags:
    - ambient
---

# How should we scope the A4 (inline) increment

## Context and Problem Statement

Question category: A4 scope.

## Considered Options

- **Emphasis only (A4a)** — Just bold (**x** / __x__) and italic (*x* / _x_) within statement text and argument descriptions. Pure text decoration, zero cross-element coupling — the smallest clean slice, matching the A2a/A2b split precedent. Defer links, mentions, and tags to later inline sub-increments.
- **Emphasis + links + tags** — Bold, italic, links ([text](url) → text + url), and inline tags (#tag). All self-contained inline elements (no cross-element references). Defer statement/argument mentions (@[T], @<T>) to a later slice. Note: inline #tag is distinct from A5's {yaml} metadata blocks.
- **Full inline** — The entire inline vocabulary in one increment: bold, italic, links, statement-mentions (@[T]), argument-mentions (@<T>), and tags (#tag). Largest slice; mentions are inline cross-references (their resolution stays Layer B, but recognizing them is A4).

## Decision Outcome

Chose **Full inline**. The entire inline vocabulary in one increment: bold, italic, links, statement-mentions (@[T]), argument-mentions (@<T>), and tags (#tag). Largest slice; mentions are inline cross-references (their resolution stays Layer B, but recognizing them is A4).
