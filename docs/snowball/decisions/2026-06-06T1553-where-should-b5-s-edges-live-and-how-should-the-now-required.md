---
title: Where should B5's edges live, and how should the now-required relation-target nodes be handled
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

# Where should B5's edges live, and how should the now-required relation-target nodes be handled

## Context and Problem Statement

Question category: Architecture.

## Considered Options

- **Extend the Model (single slice)** — build_model grows to also walk relations: mint relation-target nodes (titled merged by title, plain/untitled singletons), resolve sources via an indent stack, normalize directions, dedupe by (from,to,kind), and add `edges: Vec<Edge>` + NodeId to Model. One complete aggregate, consistent with B4b's 'one Model' decision. Meaty slice.
- **Extend Model, split B5a/B5b** — Same end state, but split: B5a = relation-target node completion + edge model + top-level relation resolution; B5b = PCS-interspersed relations + dedup. Two thinner slices.
- **Standalone build_relations** — Keep the Model as-is; a separate build_relations(&Document) -> Relations produces edges and handles its own target-node identities, composed later by B6. Avoids growing build_model, but splits the node universe across two functions.

## Decision Outcome

Chose **Extend the Model (single slice)**. build_model grows to also walk relations: mint relation-target nodes (titled merged by title, plain/untitled singletons), resolve sources via an indent stack, normalize directions, dedupe by (from,to,kind), and add `edges: Vec<Edge>` + NodeId to Model. One complete aggregate, consistent with B4b's 'one Model' decision. Meaty slice.
