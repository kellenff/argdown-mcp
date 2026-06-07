---
title: How far should the Dung half go
status: accepted
date: '2026-06-06T23:45:56.604Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01HnUwmrZoUWn5o2H9zhCb6u
  supersedes: null
  tags:
    - ambient
---

# How far should the Dung half go

## Context and Problem Statement

Question category: Dung extent.

## Considered Options

- **Map + grounded extension** — Project the Model into the abstract AF (arguments + attack edges) AND compute the grounded extension (IN/OUT/UNDEC) in Rust — the A1 'Dung extensions in Rust' goal, the real end target. Larger.
- **Map only** — Produce just the node+edge AF map (arguments + attacks) for an external consumer; defer the grounded-extension solver. Smaller; stops one step short of the goal.

## Decision Outcome

Chose **Map + grounded extension**. Project the Model into the abstract AF (arguments + attack edges) AND compute the grounded extension (IN/OUT/UNDEC) in Rust — the A1 'Dung extensions in Rust' goal, the real end target. Larger.
