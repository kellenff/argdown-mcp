---
title: How should it integrate with CI
status: accepted
date: '2026-06-07T01:18:12.743Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01NFkbW9sRHdGZVok5bUo1Vq
  supersedes: null
  tags:
    - ambient
---

# How should it integrate with CI

## Context and Problem Statement

Question category: CI.

## Considered Options

- **Local / on-demand only** — Like benchmarking — commit a seed corpus, run fuzzing locally or on demand; no CI gate. Avoids nightly-in-CI and search non-determinism.
- **+ CI smoke-run** — Also add a short scheduled/PR job (nightly toolchain) that builds the targets and runs each a few seconds to prevent bit-rot. Introduces nightly into CI.

## Decision Outcome

Chose **+ CI smoke-run**. Also add a short scheduled/PR job (nightly toolchain) that builds the targets and runs each a few seconds to prevent bit-rot. Introduces nightly into CI.
