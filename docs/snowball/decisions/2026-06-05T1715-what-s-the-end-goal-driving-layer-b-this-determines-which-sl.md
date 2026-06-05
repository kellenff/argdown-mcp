---
title: What's the end goal driving Layer B? This determines which slice we build first.
status: accepted
date: '2026-06-05T17:15:43.276Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_01FmvCRZ4ukmU1MAigs8ukjo
  supersedes: null
  tags:
    - ambient
---

# What's the end goal driving Layer B? This determines which slice we build first.

## Context and Problem Statement

Question category: First slice.

## Considered Options

- **Foundational, start at B1 Sections** — Build incrementally from the smallest, zero-dependency piece (section nesting), mirroring how A1 was the 'spine' first slice. Lowest risk, establishes the Layer-B crate/module boundary and patterns. We design B1 now; later slices follow.
- **Critical path to the argument graph** — Your real target is the dialectical graph / dung extensions. That critical path is B3 statement model → B4 arguments → B5 relations (sections/metadata/tags are orthogonal enrichments deferred). We'd design the statement-equivalence-class model (B3) first as the foundation of the graph.
- **Reference-compatible model export** — Goal is the MCP server exposing the full model like the reference's export_json. Needs most slices eventually; we'd still start foundational (B1 or B2) but design with the full model shape in mind from the outset.
- **Metadata/YAML utility first (B2)** — Start with the long-deferred parse_metadata (element {…} + frontmatter → structured, with tags promotion). Self-contained, unblocks tags + any metadata-driven behavior, introduces the YAML dep in isolation.

## Decision Outcome

Chose **Foundational, start at B1 Sections**. Build incrementally from the smallest, zero-dependency piece (section nesting), mirroring how A1 was the 'spine' first slice. Lowest risk, establishes the Layer-B crate/module boundary and patterns. We design B1 now; later slices follow.
