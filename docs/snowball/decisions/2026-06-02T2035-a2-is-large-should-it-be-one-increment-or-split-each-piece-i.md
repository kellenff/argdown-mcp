---
title: A2 is large. Should it be one increment or split? (Each piece is still its own spec→plan→build.)
status: accepted
date: '2026-06-02T20:35:36.185Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01YEezU7WqoJTuuYszku3zJ7
  supersedes: null
  tags:
    - ambient
---

# A2 is large. Should it be one increment or split? (Each piece is still its own spec→plan→build.)

## Context and Problem Statement

Question category: A2 scope.

## Considered Options

- **Split: A2a relations first** — A2a = statement references (`[T]`) + nested relations between statements (all operators/directions/recursion). A2b = arguments (`<T>`/`<T>: desc`) as blocks and relation targets. Isolates the hard indentation-nesting machinery with statements only, then layers arguments on top. (Recommended)
- **One increment: full A2** — References + arguments + nested relations together. More complete in one pass, but a much larger spec/plan with the riskiest feature (indentation nesting) entangled with argument parsing.
- **Split differently: arguments first** — A2a = arguments + references as new block kinds (no relations yet). A2b = nested relations. Gets the simpler block types in first, defers all relation machinery.

## Decision Outcome

Chose **Split differently: arguments first**. A2a = arguments + references as new block kinds (no relations yet). A2b = nested relations. Gets the simpler block types in first, defers all relation machinery.
