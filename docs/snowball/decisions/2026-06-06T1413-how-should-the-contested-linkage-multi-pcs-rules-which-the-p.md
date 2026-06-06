---
title: >-
  How should the contested linkage / multi-PCS rules (which the panel flagged as 'pending @argdown/core evidence') be
  settled
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

# How should the contested linkage / multi-PCS rules (which the panel flagged as 'pending @argdown/core evidence') be settled

## Context and Problem Statement

Question category: Reference probe.

## Considered Options

- **Probe @argdown/core, then spec** — Use the argdown MCP (export_json) on test docs to confirm real linkage, role, interspersed-relation, and multi-PCS behavior before writing the spec. Matches the project's 'track the reference' convention.
- **Adopt panel defaults** — Go with first-PCS-wins + the panel's skip-set without probing; note them as revisitable. Faster, but may diverge from @argdown/core.
- **Decide during spec review** — Write the spec with the panel defaults as placeholders and resolve the contested points when reviewing it.

## Decision Outcome

Chose **Probe @argdown/core, then spec**. Use the argdown MCP (export_json) on test docs to confirm real linkage, role, interspersed-relation, and multi-PCS behavior before writing the spec. Matches the project's 'track the reference' convention.
