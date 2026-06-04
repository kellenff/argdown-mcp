---
title: After the CLOSING fence — does the next line need to be blank/EOF
status: accepted
date: '2026-06-04T07:06:17.997Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: bd91c155-11f1-49bc-9671-18dae30b53ed
  source_event_id: toolu_01QezNSrYnUcvTTEiwWFY91B
  supersedes: null
  tags:
    - ambient
---

# After the CLOSING fence — does the next line need to be blank/EOF

## Context and Problem Statement

Question category: Trailing blank.

## Considered Options

- **No — fence self-terminates (lenient)** — The closing fence ends frontmatter; normal trivia + block parsing resumes immediately, so `===\n...\n===\n[S]` (no blank line) is accepted. Diverges from argdown's blank-line-required quirk, but it's a tokenizer artifact, not meaningful syntax. Simpler. Recommended.
- **Yes — require blank line / EOF (mirror argdown)** — Require a blank line (or EOF) after the closing fence, else error — faithful to argdown, including its demand for a trailing blank even when frontmatter is the whole document.

## Decision Outcome

Chose **Yes — require blank line / EOF (mirror argdown)**. Require a blank line (or EOF) after the closing fence, else error — faithful to argdown, including its demand for a trailing blank even when frontmatter is the whole document.
