---
title: >-
  How should the parser represent relation nesting in the AST? (This is the cross-cutting call we deferred when
  splitting A2.)
status: accepted
date: '2026-06-02T21:56:10.990Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01FusURcmdiVfvmg7ADvBQGL
  supersedes: null
  tags:
    - ambient
---

# How should the parser represent relation nesting in the AST? (This is the cross-cutting call we deferred when splitting A2.)

## Context and Problem Statement

Question category: Relation AST.

## Considered Options

- **Nested tree** — Statement/Argument gain `relations: Vec<Relation>`; each Relation has operator, direction, a target Element (Statement|Argument), and its own `relations` children. The indentation structure becomes the tree directly. Natural for consumers; recursion mirrors the source. (Recommended)
- **Flat with depth** — Parser emits a flat list of relation lines, each tagged with indent depth + operator + target; the tree is assembled later in Layer B (the semantic model). Keeps the parser flatter, consistent with how A1 deferred section nesting, but relation structure isn't usable until Layer B.

## Decision Outcome

Chose **Flat with depth**. Parser emits a flat list of relation lines, each tagged with indent depth + operator + target; the tree is assembled later in Layer B (the semantic model). Keeps the parser flatter, consistent with how A1 deferred section nesting, but relation structure isn't usable until Layer B.
