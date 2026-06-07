---
title: >-
  MCP `parse` never "fails" — a malformed document comes back as a normal result with `{ok:false, diagnostic}`. But your
  CLI spec says non-zero exit + stderr diagnostics signal errors. How should the CLI reconcile these when input fails to
  parse
status: accepted
date: '2026-06-07T16:11:46.288Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01UrVqevJRjheAgSLuPFiQSY
  supersedes: null
  tags:
    - ambient
---

# MCP `parse` never "fails" — a malformed document comes back as a normal result with `{ok:false, diagnostic}`. But your CLI spec says non-zero exit + stderr diagnostics signal errors. How should the CLI reconcile these when input fails to parse

## Context and Problem Statement

Question category: Error model.

## Considered Options

- **Unix-native** — Any parse failure (all 3 subcommands) → diagnostic with byte offset on stderr, non-zero exit. Success → result on stdout, exit 0. `parse` effectively becomes a validator. Output is NOT byte-identical to the MCP {ok:false} payload, but it's the most idiomatic CLI.
- **Hybrid** — Always print the structured result JSON to stdout (including {ok:false} for parse), AND mirror the human diagnostic to stderr, AND set non-zero exit on failure. Machine-readable stdout + script-friendly exit codes, at the cost of emitting on both streams.
- **Byte-parity** — Mirror MCP exactly: `parse` always exits 0 and prints {ok, summary|diagnostic} to stdout (it never "errors"); only export/dung exit non-zero on parse failure. Maximizes fidelity to the MCP payloads, least idiomatic as a CLI.

## Decision Outcome

Chose **Unix-native**. Any parse failure (all 3 subcommands) → diagnostic with byte offset on stderr, non-zero exit. Success → result on stdout, exit 0. `parse` effectively becomes a validator. Output is NOT byte-identical to the MCP {ok:false} payload, but it's the most idiomatic CLI.
