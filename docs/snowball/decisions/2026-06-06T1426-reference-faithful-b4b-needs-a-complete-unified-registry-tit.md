---
title: >-
  Reference-faithful B4b needs a complete unified registry (titled+untitled statements, named+anonymous arguments,
  merged by title). Where should that completion live
status: accepted
date: '2026-06-06T14:26:13.396Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01EsmLzzBachLhtKn1q5Jyro
  supersedes: null
  tags:
    - ambient
---

# Reference-faithful B4b needs a complete unified registry (titled+untitled statements, named+anonymous arguments, merged by title). Where should that completion live

## Context and Problem Statement

Question category: Registry arch.

## Considered Options

- **B4b introduces Model aggregate** — B4b consumes B3 Statements + B4a Arguments and produces the first `Model` aggregate that completes them (adds PCS-internal titled statements merged by title, untitled singletons, anonymous arguments) alongside resolved PCS. B3/B4a stay untouched. Additive; matches B3's 'B4 introduces the aggregate' note. Larger B4b.
- **Revise B3 + B4a to be complete** — Re-open the two shipped slices so B3 also registers titled statements found in PCSs (merged by title) and B4a mints anonymous arguments. Then B4b stays thin. Single clean registry per entity, but churns recently-shipped code.
- **Split into B4b + B4c** — B4b resolves PCS internals only (roles, inference, titled->B3 where it exists, composite ids otherwise); a later B4c builds the complete unified registry + anonymous args before B5. Keeps each slice thin.

## Decision Outcome

Chose **B4b introduces Model aggregate**. B4b consumes B3 Statements + B4a Arguments and produces the first `Model` aggregate that completes them (adds PCS-internal titled statements merged by title, untitled singletons, anonymous arguments) alongside resolved PCS. B3/B4a stay untouched. Additive; matches B3's 'B4 introduces the aggregate' note. Larger B4b.
