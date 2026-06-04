---
title: How should A5 represent metadata content
status: accepted
date: '2026-06-04T02:12:46.128Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_01CLtcWVYsycMoxsbyQ5LNfJ
  supersedes: null
  tags:
    - ambient
---

# How should A5 represent metadata content

## Context and Problem Statement

Question category: Metadata repr.

## Considered Options

- **B: raw string + span, stripped** — Metadata { raw: String, span: Span } on each site (statement/argument/heading/inference) + Document.frontmatter; the {...} block is stripped from the element's text (metadata is typed data, not prose). Brace-matched (depth/quote aware) but NOT YAML-parsed; no serde_yaml dependency. The single canonical parse_metadata utility + error enum is deferred to Layer B. (M2-jam recommendation.)
- **A: span only, stripped** — Metadata { span: Span } — capture only the source range; no raw string. Strictly minimal, spans-only (most A4-consistent). But every consumer must re-read source[span] AND each runs its own YAML parse — the 'distributed liability' the jam flagged.
- **C: parse YAML now** — Metadata { data: <yaml value> } via a serde_yaml dependency at parse time — structured key/values immediately, matching the reference. Heaviest: adds a dependency, the recognizer now interprets values, a new malformed-YAML error surface, and it's premature (no consumer needs structured data yet).

## Decision Outcome

Chose **B: raw string + span, stripped**. Metadata { raw: String, span: Span } on each site (statement/argument/heading/inference) + Document.frontmatter; the {...} block is stripped from the element's text (metadata is typed data, not prose). Brace-matched (depth/quote aware) but NOT YAML-parsed; no serde_yaml dependency. The single canonical parse_metadata utility + error enum is deferred to Layer B. (M2-jam recommendation.)
