---
title: >-
  What's driving the idea of replacing Argdown with YAML? (Select all that genuinely apply — this is what the
  recommendation hinges on.)
status: accepted
date: '2026-06-07T04:38:35.665Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: bc5a833b-097c-48ac-a169-d40e57bfe591
  source_event_id: toolu_01SYqBDm49AdxLRpF9p9FtZV
  supersedes: null
  tags:
    - ambient
---

# What's driving the idea of replacing Argdown with YAML? (Select all that genuinely apply — this is what the recommendation hinges on.)

## Context and Problem Statement

Question category: The driver.

## Considered Options

- **Parser maintenance burden** — The bespoke recognizer — plus fuzzing, benches, byte-span discipline — is a lot of code to own and harden. YAML parsers are free and battle-tested; you'd delete a large pile of parsing code.
- **Agents can't emit Argdown** — The target consumer is LLM agents. Terse prose-markup is error-prone for them to *produce* reliably; structured YAML is easier for a model to generate and self-validate.
- **Reference compat isn't used** — @argdown/core interop and the upstream ecosystem (VS Code, argdown.org files) aren't actually consumed by anyone here; the fidelity target is academic overhead you'd happily shed.
- **Want schema & tooling** — You want JSON-Schema validation, off-the-shelf editor support, and unambiguous structure — things YAML/JSON give for free that the bespoke grammar doesn't.

## Decision Outcome

Chose **Parser maintenance burden**. The bespoke recognizer — plus fuzzing, benches, byte-span discipline — is a lot of code to own and harden. YAML parsers are free and battle-tested; you'd delete a large pile of parsing code.
