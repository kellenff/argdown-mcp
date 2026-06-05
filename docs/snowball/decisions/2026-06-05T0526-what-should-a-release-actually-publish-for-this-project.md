---
title: What should a "release" actually publish for this project
status: accepted
date: '2026-06-05T05:26:12.137Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: c6d6d1c0-4521-4566-81c9-1253590e1bb5
  source_event_id: toolu_018U8oTQNJjKaWAeS5s9t1u4
  supersedes: null
  tags:
    - ambient
---

# What should a "release" actually publish for this project

## Context and Problem Statement

Question category: Artifact.

## Considered Options

- **GitHub Release + binaries** — Cross-compile the argdown-mcp binary (Linux/macOS/Windows), attach the binaries to a GitHub Release with auto-generated notes. Best fit for distributing an MCP server people run locally.
- **Publish to crates.io** — cargo publish all three crates (core, parser, mcp) to the registry. Requires a CRATES_IO_TOKEN secret and crates to be publish-ready (no path-only deps, metadata filled in).
- **Both binaries + crates.io** — GitHub Release with binaries AND cargo publish to crates.io. Most complete, most moving parts to maintain.
- **GitHub Release only (no binaries)** — Just tag + create a GitHub Release marker from source, no compiled artifacts. Lightest; users build from source.

## Decision Outcome

Chose **GitHub Release + binaries**. Cross-compile the argdown-mcp binary (Linux/macOS/Windows), attach the binaries to a GitHub Release with auto-generated notes. Best fit for distributing an MCP server people run locally.
