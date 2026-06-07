---
title: >-
  Where should YAML/JSON export be exposed? (The argdown-mcp binary is currently just a placeholder that calls parse —
  no MCP SDK is wired up yet.)
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

# Where should YAML/JSON export be exposed? (The argdown-mcp binary is currently just a placeholder that calls parse — no MCP SDK is wired up yet.)

## Context and Problem Statement

Question category: Surface.

## Considered Options

- **Rust library API** — Add export functions (e.g. to_json/to_yaml) in argdown-model returning Strings. Foundational — everything else builds on this. Lowest effort, fully unit-testable.
- **CLI on the binary** — Turn the argdown-mcp binary into a runnable CLI: read source from stdin/file, print JSON or YAML. Makes export usable & demoable today without an MCP SDK.
- **Real MCP tools** — Wire an MCP SDK (e.g. rmcp) and expose export_json/export_yaml as MCP tools. Largest effort — pulls in async runtime + protocol layer that doesn't exist yet.

## Decision Outcome

Chose **Rust library API**. Add export functions (e.g. to_json/to_yaml) in argdown-model returning Strings. Foundational — everything else builds on this. Lowest effort, fully unit-testable.
