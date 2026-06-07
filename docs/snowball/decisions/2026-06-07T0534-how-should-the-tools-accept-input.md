---
title: How should the tools accept input
status: accepted
date: '2026-06-07T05:34:01.930Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_01D8M9NAeNwHMMcMXFyQ2Bxq
  supersedes: null
  tags:
    - ambient
---

# How should the tools accept input

## Context and Problem Statement

Question category: Input mode.

## Considered Options

- **Inline only** — Each tool takes a `source` string. No filesystem access at all — simplest schema, no permissions to reason about, matches the reference's preferred mode.
- **Inline + file path** — Also accept a `path` to read from disk. Convenient when driving the server over a local document, but the binary then needs filesystem read access (a capability/security surface).

## Decision Outcome

Chose **Inline only**. Each tool takes a `source` string. No filesystem access at all — simplest schema, no permissions to reason about, matches the reference's preferred mode.
