---
title: How much of the PCS grammar should A3 cover
status: accepted
date: '2026-06-03T03:26:22.034Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_015a2XLzKZKbukVtuP3Mepzc
  supersedes: null
  tags:
    - ambient
---

# How much of the PCS grammar should A3 cover

## Context and Problem Statement

Question category: A3 scope.

## Considered Options

- **Structure + rule names** — A3 parses numbered statement lines, both inference-line forms (bare `----` and `-- Rule, Rule --`) capturing rule NAMES, and child relations on PCS statements. The `{yaml}` metadata block (on inference lines and elsewhere) defers to A5. Aligns with the roadmap's own A5=metadata seam and the thin-slice philosophy.
- **Full PCS incl. metadata** — A3 also parses the `{yaml}` metadata block on inference lines now (rule names + structured/raw metadata). One increment covers the entire inference-line surface, but pulls metadata handling forward out of A5 and introduces YAML-ish parsing earlier.
- **Skeleton only** — A3 parses numbered statement lines + the bare `----` divider only. ALL inference annotations (rule names AND metadata) defer to a later increment. Smallest slice, but `-- Modus Ponens --` would be a parse error for now.

## Decision Outcome

Chose **Structure + rule names**. A3 parses numbered statement lines, both inference-line forms (bare `----` and `-- Rule, Rule --`) capturing rule NAMES, and child relations on PCS statements. The `{yaml}` metadata block (on inference lines and elsewhere) defers to A5. Aligns with the roadmap's own A5=metadata seam and the thin-slice philosophy.
