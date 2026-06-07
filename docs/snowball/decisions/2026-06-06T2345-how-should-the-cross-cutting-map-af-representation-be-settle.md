---
title: How should the cross-cutting map/AF representation be settled
status: accepted
date: '2026-06-06T23:45:56.604Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01HnUwmrZoUWn5o2H9zhCb6u
  supersedes: null
  tags:
    - ambient
---

# How should the cross-cutting map/AF representation be settled

## Context and Problem Statement

Question category: Process.

## Considered Options

- **Chorus brain-jam + probe** — Pressure-test the tag-registry + AF/extension representation with a multi-model chorus before the spec, plus more @argdown/core probes for contradictory/undercut/tag-promotion edge cases. The proven B5 path.
- **I design + review** — The AF projection is now well-constrained by the reference probes; I design it directly from the probes + Model, write the spec, TDD-build, and run the two-stage review. Faster.

## Decision Outcome

Chose **Chorus brain-jam + probe**. Pressure-test the tag-registry + AF/extension representation with a multi-model chorus before the spec, plus more @argdown/core probes for contradictory/undercut/tag-promotion edge cases. The proven B5 path.
