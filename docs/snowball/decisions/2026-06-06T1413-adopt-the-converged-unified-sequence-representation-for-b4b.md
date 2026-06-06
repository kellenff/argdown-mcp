---
title: Adopt the converged unified-sequence representation for B4b
status: accepted
date: '2026-06-06T14:13:52.596Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01QcB6zXP9LeGjwpZbfu1ABP
  supersedes: null
  tags:
    - ambient
---

# Adopt the converged unified-sequence representation for B4b

## Context and Problem Statement

Question category: Representation.

## Considered Options

- **Adopt as-is** — Lock the unified Vec<ResolvedPcsItem> design + Pcss/argument_to_pcs + issues-as-data + build_pcs(&Document,&Arguments,&Statements) as the B4b spec basis.
- **Adopt with changes** — Mostly good, but I want to adjust something (e.g. keep premises pre-computed, or a different aggregate). I'll specify.
- **Discuss further** — Talk through the design more before committing — e.g. run another chorus round or weigh an alternative.

## Decision Outcome

Chose **Adopt as-is**. Lock the unified Vec<ResolvedPcsItem> design + Pcss/argument_to_pcs + issues-as-data + build_pcs(&Document,&Arguments,&Statements) as the B4b spec basis.
