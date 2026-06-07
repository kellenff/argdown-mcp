---
title: >-
  A Claude Code plugin launches an MCP server by running a command on the user's machine. The argdown server is a
  compiled Rust binary. How should the plugin obtain/launch it
status: accepted
date: '2026-06-07T17:32:18.147Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_017CPtcrzKrVAmBVcqa1np97
  supersedes: null
  tags:
    - ambient
---

# A Claude Code plugin launches an MCP server by running a command on the user's machine. The argdown server is a compiled Rust binary. How should the plugin obtain/launch it

## Context and Problem Statement

Question category: Launch mechanism.

## Considered Options

- **Download prebuilt binary** — A self-bootstrapping launcher (the MCP `command` itself) downloads the matching argdown-mcp-v0.1.0-<target> archive from the GitHub Release on first run, verifies it against SHA256SUMS, caches it under the plugin dir, then execs it. Zero Rust toolchain, 'just works', and it's exactly what release.yml exists for. Cost: a cross-platform download+verify launcher script (the real complexity is Windows vs POSIX).
- **Build from source (cargo)** — The plugin ships this repo's source; `.mcp.json` runs `cargo run --release -p argdown-mcp`. Always matches the code, simplest packaging, no download logic. Cost: requires a Rust toolchain on the user's machine and a slow first-run compile; plugin carries the whole workspace.
- **Expect argdown-mcp on PATH** — User installs the binary separately (download a release or `cargo install`); the plugin just registers the MCP server (`command: argdown-mcp`) plus optional usage docs. Leanest plugin, no binary management at all. Cost: it doesn't 'just work' on install — the user must place the binary first.

## Decision Outcome

Chose **Download prebuilt binary**. A self-bootstrapping launcher (the MCP `command` itself) downloads the matching argdown-mcp-v0.1.0-<target> archive from the GitHub Release on first run, verifies it against SHA256SUMS, caches it under the plugin dir, then execs it. Zero Rust toolchain, 'just works', and it's exactly what release.yml exists for. Cost: a cross-platform download+verify launcher script (the real complexity is Windows vs POSIX).
