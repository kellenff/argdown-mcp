---
title: >-
  Beyond registering the MCP server itself, should the plugin bundle anything to help Claude actually use the three
  tools (parse / export_model / dung_extensions)
status: accepted
date: '2026-06-07T17:34:14.268Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01VkArZ4541tzDAbVjAVpV25
  supersedes: null
  tags:
    - ambient
---

# Beyond registering the MCP server itself, should the plugin bundle anything to help Claude actually use the three tools (parse / export_model / dung_extensions)

## Context and Problem Statement

Question category: Plugin scope.

## Considered Options

- **MCP server only** — The plugin registers just the MCP server — the three tools appear and Claude calls them when relevant. Leanest, true to 'package the MCP server', YAGNI. Discovery relies on the tools' own descriptions (which are already written).
- **+ a usage skill** — Add one small skill that tells Claude when to reach for the argdown tools (e.g. 'when analyzing argument structure / dialectical maps, parse with argdown and compute the grounded extension'). Improves triggering and teaches the workflow, at the cost of one more component to maintain.
- **+ skill and a slash command** — The usage skill plus a convenience slash command (e.g. /argdown:dung over a pasted/selected document). Most batteries-included, but the most surface to build and maintain for a v0.1 packaging.

## Decision Outcome

Chose **+ skill and a slash command**. The usage skill plus a convenience slash command (e.g. /argdown:dung over a pasted/selected document). Most batteries-included, but the most surface to build and maintain for a v0.1 packaging.
