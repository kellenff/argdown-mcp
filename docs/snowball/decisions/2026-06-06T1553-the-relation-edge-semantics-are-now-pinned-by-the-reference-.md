---
title: The relation edge semantics are now pinned by the reference probes. How should the representation be settled
status: accepted
date: '2026-06-06T15:53:55.080Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01KuTtsvWQfseERd7bRGy4qX
  supersedes: null
  tags:
    - ambient
---

# The relation edge semantics are now pinned by the reference probes. How should the representation be settled

## Context and Problem Statement

Question category: Process.

## Considered Options

- **Design directly + review** — Skip the brain-jam: the edge/node representation is now well-constrained by the reference probes + the B4b Model template, so I design it directly, write the spec, TDD-build, and run the two-stage review. Faster.
- **Chorus brain-jam first** — Pressure-test the edge/node representation (NodeId shape, dedup, contradictory, source-resolution data model) with a multi-model chorus brain-jam before the spec, as we did for B4b. More thorough.

## Decision Outcome

Chose **Chorus brain-jam first**. Pressure-test the edge/node representation (NodeId shape, dedup, contradictory, source-resolution data model) with a multi-model chorus brain-jam before the spec, as we did for B4b. More thorough.
