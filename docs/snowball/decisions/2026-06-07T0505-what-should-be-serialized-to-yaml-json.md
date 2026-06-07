---
title: What should be serialized to YAML/JSON
status: accepted
date: '2026-06-07T05:05:46.972Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_01JWEgS9b4NEvVgDMNZkkY9T
  supersedes: null
  tags:
    - ambient
---

# What should be serialized to YAML/JSON

## Context and Problem Statement

Question category: What to export.

## Considered Options

- **Layer B Model** — The rich semantic model: statement/argument registries, resolved PCS with roles, dialectical edges, conflicts/issues. The 'real' output of the toolchain.
- **Layer A Document (AST)** — The raw parse tree (blocks, frontmatter, inlines, spans). Closest analog to @argdown/core's export_json shape.
- **Dung outputs** — ArgumentationFramework + GroundedLabelling (IN/OUT/UNDEC). Small, derived from the Model. Mirrors the dung_extensions tool.

## Decision Outcome

Chose **Layer B Model**. The rich semantic model: statement/argument registries, resolved PCS with roles, dialectical edges, conflicts/issues. The 'real' output of the toolchain.
