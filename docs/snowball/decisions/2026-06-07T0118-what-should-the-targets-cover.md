---
title: What should the targets cover
status: accepted
date: '2026-06-07T01:18:12.743Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01NFkbW9sRHdGZVok5bUo1Vq
  supersedes: null
  tags:
    - ambient
---

# What should the targets cover

## Context and Problem Statement

Question category: Scope.

## Considered Options

- **Full pipeline** — Fuzz parse(), and on Ok also run Layer B (build_model / build_tags / dung_framework + grounded_extension). Assert no panics + invariants (spans in source bounds; AF well-formed). Exercises the 'Layer B is total' claim end-to-end.
- **Parser only** — Fuzz parse() alone; assert it never panics and every span is within source bounds. Smaller; the parser is the stated public boundary.

## Decision Outcome

Chose **Full pipeline**. Fuzz parse(), and on Ok also run Layer B (build_model / build_tags / dung_framework + grounded_extension). Assert no panics + invariants (spans in source bounds; AF well-formed). Exercises the 'Layer B is total' claim end-to-end.
