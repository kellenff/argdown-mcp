---
title: How robust should A1 parsing be
status: accepted
date: '2026-06-02T18:57:01.022Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_0116iDrZ3fHZigZTVqg5qiqi
  supersedes: null
  tags:
    - ambient
---

# How robust should A1 parsing be

## Context and Problem Statement

Question category: Robustness.

## Considered Options

- **Strict fail-fast** — parse() returns Result<Document, Error> and stops at the first syntax error with a precise message + span. Error-recovery (partial AST + diagnostic list, like the reference) is a later increment. (Recommended)
- **Recoverable now** — Collect diagnostics and produce a partial AST on errors, matching the reference's lenient behavior. More machinery up front (error nodes, resync points).

## Decision Outcome

Chose **Strict fail-fast**. parse() returns Result<Document, Error> and stops at the first syntax error with a precise message + span. Error-recovery (partial AST + diagnostic list, like the reference) is a later increment. (Recommended)
