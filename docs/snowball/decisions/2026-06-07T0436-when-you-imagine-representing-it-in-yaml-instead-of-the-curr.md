---
title: >-
  When you imagine 'representing it in YAML instead of the current Argdown format,' which of these is closest to what
  you're weighing
status: accepted
date: '2026-06-07T04:36:52.191Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: bc5a833b-097c-48ac-a169-d40e57bfe591
  source_event_id: toolu_01RtP7AJmgzB1Xiqge86Y8tw
  supersedes: null
  tags:
    - ambient
---

# When you imagine 'representing it in YAML instead of the current Argdown format,' which of these is closest to what you're weighing

## Context and Problem Statement

Question category: What changes.

## Considered Options

- **Replace Argdown entirely** — YAML becomes the canonical authoring format; drop the bespoke parser and, with it, @argdown/core reference compatibility and the upstream ecosystem. The project stops being 'an Argdown implementation.'
- **Add YAML alongside** — Keep Argdown as the canonical format and reference target, but add a YAML representation as a second input/output surface — e.g. a structured exchange format for agents to emit/consume.
- **Agent-facing I/O only** — Argdown stays the format; the concern is specifically the MCP surface — let agents read/write a structured (YAML/JSON) view of the model, rather than hand-writing terse markup.
- **Pressure-test the foundation** — Step back and genuinely re-examine whether the bespoke-Argdown-parser bet was the right one at all — open to any conclusion, including 'no.'

## Decision Outcome

Chose **Replace Argdown entirely**. YAML becomes the canonical authoring format; drop the bespoke parser and, with it, @argdown/core reference compatibility and the upstream ecosystem. The project stops being 'an Argdown implementation.'
