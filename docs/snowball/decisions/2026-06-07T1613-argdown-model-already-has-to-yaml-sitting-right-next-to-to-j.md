---
title: >-
  `argdown-model` already has `to_yaml` sitting right next to `to_json`, but the MCP `export_model` tool only surfaces
  JSON. Your recent MADR (2026-06-07) was specifically about where YAML/JSON export should be exposed. For the CLI's
  export subcommand, which output formats
status: accepted
date: '2026-06-07T16:13:10.168Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01GjDUcdd6bGEjPFT7c2xe4x
  supersedes: null
  tags:
    - ambient
---

# `argdown-model` already has `to_yaml` sitting right next to `to_json`, but the MCP `export_model` tool only surfaces JSON. Your recent MADR (2026-06-07) was specifically about where YAML/JSON export should be exposed. For the CLI's export subcommand, which output formats

## Context and Problem Statement

Question category: Output fmt.

## Considered Options

- **JSON only (strict parity)** — Match the MCP exactly: `export` emits pretty JSON, no format flag. Smallest surface, true feature parity, honors YAGNI. The latent to_yaml stays unexposed until separately requested — keeps this work tightly scoped to 'mirror the MCP'.
- **Add --format json|yaml** — `export` gains a `--format` flag (default json) that surfaces the existing to_yaml at near-zero cost. Slightly exceeds MCP parity, but the CLI becomes the natural home for the YAML surface your MADR was weighing. One flag, one match arm.

## Decision Outcome

Chose **Add --format json|yaml**. `export` gains a `--format` flag (default json) that surfaces the existing to_yaml at near-zero cost. Slightly exceeds MCP parity, but the CLI becomes the natural home for the YAML surface your MADR was weighing. One flag, one match arm.
