---
title: >-
  How faithfully should B4b reproduce the reference's auto-naming of standalone PCS (anonymous arguments) and untitled
  PCS statements
status: accepted
date: '2026-06-06T14:22:12.007Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01GhGczqCmJq7ERHAyAaM3pH
  supersedes: null
  tags:
    - ambient
---

# How faithfully should B4b reproduce the reference's auto-naming of standalone PCS (anonymous arguments) and untitled PCS statements

## Context and Problem Statement

Question category: B4b fidelity.

## Considered Options

- **Minimal / defer** — B4b records structural facts only: argument_id: Option<ArgumentId> (None = standalone) and statement_id: Option<StatementId> (None = untitled). No minting of anonymous-arg or untitled-statement identities — deferred to the consumer (B6) or a later B4a/B3 revision. Thinnest slice, consistent with B1-B4a minimalism.
- **Mint anonymous args only** — B4b gives each standalone PCS an owning anonymous-argument identity (so every PCS has an argument node for B5/B6), but leaves untitled PCS statements as None. Middle ground.
- **Reference-faithful (full)** — B4b mints both anonymous arguments and untitled-statement equivalence classes, matching @argdown/core's model exactly. Largest slice; expands B4a's argument space and B3's statement space.

## Decision Outcome

Chose **Reference-faithful (full)**. B4b mints both anonymous arguments and untitled-statement equivalence classes, matching @argdown/core's model exactly. Largest slice; expands B4a's argument space and B3's statement space.
