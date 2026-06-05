---
title: What should the benchmark inputs (the corpus) be
status: accepted
date: '2026-06-05T06:28:54.638Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_01DtuZMS7uTyzLj9STTBeTGA
  supersedes: null
  tags:
    - ambient
---

# What should the benchmark inputs (the corpus) be

## Context and Problem Statement

Question category: Corpus.

## Considered Options

- **Feature micros + size scaling** — One small input per grammar construct (heading, statement, relation, PCS, inline, metadata, frontmatter) so a regression points at the exact recognizer, PLUS a representative mixed document at 2–3 sizes (small/medium/large) to track end-to-end throughput and scaling. Criterion groups make this cheap. Best regression-guard granularity. (Recommended.)
- **Mixed representative docs only** — A few hand-written Argdown documents exercising the full grammar mix, at small/medium/large sizes. Measures parse() end-to-end and scaling. Leanest start; you'd know parsing slowed but not which recognizer.
- **Real-world corpus** — Commit real .argdown files (e.g. from the upstream argdown examples) as fixtures. Most realistic load profile, but adds sourcing/licensing overhead and fixed, uncontrolled sizes.

## Decision Outcome

Chose **Feature micros + size scaling**. One small input per grammar construct (heading, statement, relation, PCS, inline, metadata, frontmatter) so a regression points at the exact recognizer, PLUS a representative mixed document at 2–3 sizes (small/medium/large) to track end-to-end throughput and scaling. Criterion groups make this cheap. Best regression-guard granularity. (Recommended.)
