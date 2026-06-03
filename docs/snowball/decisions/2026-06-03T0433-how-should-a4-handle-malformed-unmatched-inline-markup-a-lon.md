---
title: How should A4 handle malformed/unmatched inline markup (a lone `*`, an unclosed `**`, a `[` with no `](url)`)
status: accepted
date: '2026-06-03T04:33:45.771Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_01UwMNKAeS26H4yYNNza7oCf
  supersedes: null
  tags:
    - ambient
---

# How should A4 handle malformed/unmatched inline markup (a lone `*`, an unclosed `**`, a `[` with no `](url)`)

## Context and Problem Statement

Question category: Inline errors.

## Considered Options

- **Forgiving (literal text)** — Unmatched or malformed inline markup is just literal text — no emphasis/link recognized, no parse error. Matches the reference and markdown norms. Block-level structure (references, relation operators, PCS markers, inference lines) stays strict fail-fast; only inline is loose, because emphasis markup is inherently ambiguous in prose. (Reference behavior; my recommendation.)
- **Strict (parse error)** — Malformed inline (unclosed **, link without a destination) is a hard Err{message, offset}, consistent with the project's block-level strict fail-fast. Uniform error model everywhere, but diverges from the reference and would reject prose that happens to contain a stray * or [ as a literal character.

## Decision Outcome

Chose **Strict (parse error)**. Malformed inline (unclosed **, link without a destination) is a hard Err{message, offset}, consistent with the project's block-level strict fail-fast. Uniform error model everywhere, but diverges from the reference and would reject prose that happens to contain a stray * or [ as a literal character.
