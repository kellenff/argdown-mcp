---
title: Which crate-placement approach should the design use
status: accepted
date: '2026-06-07T16:16:43.521Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_017uGasHKUvAeTPDjmUYGNYf
  supersedes: null
  tags:
    - ambient
---

# Which crate-placement approach should the design use

## Context and Problem Statement

Question category: Crate layout.

## Considered Options

- **A: Extract argdown-tools** — New shared argdown-tools crate (pure core) + new argdown-cli crate. Ports-and-adapters faithful; CLI stays free of rmcp/tokio; MCP crate shrinks to its adapter. Costs a contained refactor + schemars feature-gate. Recommended.
- **B: Second bin in argdown-mcp** — Add the CLI as a [[bin]] inside the existing argdown-mcp crate. Smallest diff, but the 'mcp' crate becomes a grab-bag that ships a non-MCP CLI dragging rmcp/tokio.
- **C: argdown-cli depends on argdown-mcp** — Separate CLI crate, but reaches pure logic through the whole MCP server crate — backwards dependency direction, compiles rmcp/tokio needlessly.

## Decision Outcome

Chose **A: Extract argdown-tools**. New shared argdown-tools crate (pure core) + new argdown-cli crate. Ports-and-adapters faithful; CLI stays free of rmcp/tokio; MCP crate shrinks to its adapter. Costs a contained refactor + schemars feature-gate. Recommended.
