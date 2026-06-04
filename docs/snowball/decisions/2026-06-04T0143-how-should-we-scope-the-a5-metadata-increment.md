---
title: How should we scope the A5 (metadata) increment
status: accepted
date: '2026-06-04T01:43:12.062Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_01SoWHiNLEehQeGDiFA85ucb
  supersedes: null
  tags:
    - ambient
---

# How should we scope the A5 (metadata) increment

## Context and Problem Statement

Question category: A5 scope.

## Considered Options

- **Full metadata** — Both features in one increment: trailing `{yaml}` metadata blocks on every site (statement, argument, heading, inference line, and relation if applicable) AND document frontmatter (=== ... ===). Matches the A4 'do the whole vocabulary' choice; one cohesive metadata increment.
- **Brace metadata only (A5a)** — Just the trailing `{yaml}` blocks on statement/argument/heading/inference. Defer document frontmatter (=== fences) to a later slice (A5b). One recognizer (a brace block) applied at several attachment sites; frontmatter is a distinct fenced-block feature.
- **Frontmatter first (A5a)** — Just document frontmatter (=== ... ===) first; defer the trailing `{yaml}` element-metadata blocks to a later slice. Frontmatter is a single document-level block; element metadata touches many sites. Smallest first slice.

## Decision Outcome

Chose **Full metadata**. Both features in one increment: trailing `{yaml}` metadata blocks on every site (statement, argument, heading, inference line, and relation if applicable) AND document frontmatter (=== ... ===). Matches the A4 'do the whole vocabulary' choice; one cohesive metadata increment.
