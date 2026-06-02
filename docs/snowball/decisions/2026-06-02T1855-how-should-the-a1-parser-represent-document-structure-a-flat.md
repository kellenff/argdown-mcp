---
title: 'How should the A1 parser represent document structure: a flat block sequence, or a nested section tree'
status: accepted
date: '2026-06-02T18:55:21.397Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01RpfGo3EBfnMF6u8wFc8pUK
  supersedes: null
  tags:
    - ambient
---

# How should the A1 parser represent document structure: a flat block sequence, or a nested section tree

## Context and Problem Statement

Question category: AST shape.

## Considered Options

- **Flat block sequence** — Parser emits Vec<Block> where Block = Heading{level,text,span} | Statement{..} . Section nesting and statement→section assignment become a separate Layer-B step. Keeps the parser a pure line/block recognizer; simplest to test. (Recommended)
- **Nested section tree** — Parser builds Section nodes that contain child statements and sub-sections by heading level (a level stack). Output mirrors the reference 'sections' tree directly, but folds structural assembly into the parser.

## Decision Outcome

Chose **Flat block sequence**. Parser emits Vec<Block> where Block = Heading{level,text,span} | Statement{..} . Section nesting and statement→section assignment become a separate Layer-B step. Keeps the parser a pure line/block recognizer; simplest to test. (Recommended)
