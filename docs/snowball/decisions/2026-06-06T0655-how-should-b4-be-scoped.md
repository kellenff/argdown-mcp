---
title: How should B4 be scoped
status: accepted
date: '2026-06-06T06:55:24.597Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_014xwd5uLrQcLGkgHfs2ffSE
  supersedes: null
  tags:
    - ambient
---

# How should B4 be scoped

## Context and Problem Statement

Question category: Scope.

## Considered Options

- **Split: B4a then B4b** — B4a = argument equivalence-class model (mirrors B3 closely, low risk). B4b = PCS role resolution + inference binding + argument↔PCS linkage (cross-cutting, novel). Matches the A2a/A2b and A5a/A5b split precedent.
- **Single slice B4** — Argument model + PCS roles/inference together in one spec→plan→build. Fewer ceremony steps, but a larger blast radius for the cross-cutting representation call.

## Decision Outcome

Chose **Split: B4a then B4b**. B4a = argument equivalence-class model (mirrors B3 closely, low risk). B4b = PCS role resolution + inference binding + argument↔PCS linkage (cross-cutting, novel). Matches the A2a/A2b and A5a/A5b split precedent.
