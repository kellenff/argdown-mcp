---
title: >-
  The CLI needs subcommands (parse/export/dung) plus a --format flag. The workspace is deliberately lean
  (winnow/serde/serde_json/noyalib — you even stripped criterion's default features to keep CI lean). How should the CLI
  parse arguments
status: accepted
date: '2026-06-07T16:14:06.168Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01PJDB2awGHQCs9cRzd8JnFN
  supersedes: null
  tags:
    - ambient
---

# The CLI needs subcommands (parse/export/dung) plus a --format flag. The workspace is deliberately lean (winnow/serde/serde_json/noyalib — you even stripped criterion's default features to keep CI lean). How should the CLI parse arguments

## Context and Problem Statement

Question category: Arg parsing.

## Considered Options

- **clap (derive)** — The idiomatic Rust CLI choice: derive-based subcommands, auto-generated --help/usage, polished error messages, trivially extensible. Cost: pulls clap's dependency tree into the workspace. Confined to the leaf binary crate, so the parser/model cores stay lean — lean-deps is really about keeping the *recognizer* minimal, not the leaf binary.
- **Hand-rolled minimal** — ~40 lines: match argv for the subcommand, parse the lone --format flag, read stdin. Zero new deps, maximally honors the lean-deps ADR. Cost: you write the --help text and arg-error handling by hand; gets awkward if the CLI grows more flags later.
- **lexopt (tiny)** — A near-single-file, minimal-dependency argument parser. Middle ground: real flag/subcommand parsing without clap's weight. Cost: one small dep, and you still hand-write the help text.

## Decision Outcome

Chose **clap (derive)**. The idiomatic Rust CLI choice: derive-based subcommands, auto-generated --help/usage, polished error messages, trivially extensible. Cost: pulls clap's dependency tree into the workspace. Confined to the leaf binary crate, so the parser/model cores stay lean — lean-deps is really about keeping the *recognizer* minimal, not the leaf binary.
