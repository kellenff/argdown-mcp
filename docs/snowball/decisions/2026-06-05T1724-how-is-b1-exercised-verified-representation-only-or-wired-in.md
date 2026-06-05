---
title: How is B1 exercised / verified — representation-only, or wired into the MCP server
status: accepted
date: '2026-06-05T17:24:11.281Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_012drZyt8RvRHvcFqNRE8Mc8
  supersedes: null
  tags:
    - ambient
---

# How is B1 exercised / verified — representation-only, or wired into the MCP server

## Context and Problem Statement

Question category: Consumer.

## Considered Options

- **Representation-only** — B1 builds the section model and is verified by unit tests over its output (like every prior increment). argdown-mcp stays a placeholder. The actual MCP protocol layer remains deferred as its own future effort. Smallest, consistent with the project so far. (Recommended.)
- **Wire into the MCP server** — B1 also starts the real MCP server: expose the section model (e.g., a document-outline tool) over the protocol. Much larger — pulls the deferred 'MCP protocol layer' into this slice, a separate subsystem from Layer B itself. Would itself warrant its own spec.

## Decision Outcome

Chose **Representation-only**. B1 builds the section model and is verified by unit tests over its output (like every prior increment). argdown-mcp stays a placeholder. The actual MCP protocol layer remains deferred as its own future effort. Smallest, consistent with the project so far. (Recommended.)
