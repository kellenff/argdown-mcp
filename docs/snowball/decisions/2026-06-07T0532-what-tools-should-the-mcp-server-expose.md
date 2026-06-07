---
title: What tools should the MCP server expose
status: accepted
date: '2026-06-07T05:32:52.266Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_014SA7V8rRzam62AbV5p1kJs
  supersedes: null
  tags:
    - ambient
---

# What tools should the MCP server expose

## Context and Problem Statement

Question category: Tool surface.

## Considered Options

- **Reference trio** — parse, export_json, dung_extensions — mirrors @argdown/core and the original 'D' plan. Focused MVP; all three pipeline pieces already exist (parse → build_model → dung).
- **Trio + YAML & import** — The trio plus export_yaml and a validating import tool (load a Model from JSON/YAML). Surfaces the export/import work from this session; more surface to design/test.
- **Lean (parse + export)** — Just parse + a single export tool. Drops dung_extensions for now. Smallest server; least useful for argumentation analysis.

## Decision Outcome

Chose **Reference trio**. parse, export_json, dung_extensions — mirrors @argdown/core and the original 'D' plan. Focused MVP; all three pipeline pieces already exist (parse → build_model → dung).
