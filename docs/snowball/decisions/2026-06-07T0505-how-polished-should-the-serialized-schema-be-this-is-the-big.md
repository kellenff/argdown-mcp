---
title: >-
  How polished should the serialized schema be? (This is the biggest effort/longevity lever, and intersects the paused
  'YAML-as-canonical' question.)
status: accepted
date: '2026-06-07T05:05:46.972Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_01JWEgS9b4NEvVgDMNZkkY9T
  supersedes: null
  tags:
    - ambient
---

# How polished should the serialized schema be? (This is the biggest effort/longevity lever, and intersects the paused 'YAML-as-canonical' question.)

## Context and Problem Statement

Question category: Schema style.

## Considered Options

- **Plain serde dump** — Derive Serialize on the existing types as-is (incl. ids and byte spans). Fast, idiomatic, read-only export. Shape tracks the Rust types and may change as they do.
- **Curated agent-facing schema** — Hand-design a stable, documented, span-free shape (with a schema_version field), built to round-trip. More work; partly pre-commits the YAML-canonical direction that's currently paused for the chorus debate.

## Decision Outcome

Chose **Plain serde dump**. Derive Serialize on the existing types as-is (incl. ids and byte spans). Fast, idiomatic, read-only export. Shape tracks the Rust types and may change as they do.
