---
title: >-
  How do you want to execute the plugin plan? (It runs on a feature branch + PR; post-merge steps wait on the v0.1.1
  release.)
status: accepted
date: '2026-06-07T17:58:55.444Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01UqDcSK3pUaCtqs5qjnxVEz
  supersedes: null
  tags:
    - ambient
---

# How do you want to execute the plugin plan? (It runs on a feature branch + PR; post-merge steps wait on the v0.1.1 release.)

## Context and Problem Statement

Question category: Execution.

## Considered Options

- **Subagent-Driven** — Fresh subagent per task with two-stage (spec + quality) review between tasks, on the feat/argdown-plugin branch. Recommended — the launcher bootstrap and the release.yml change each get a real review gate.
- **Inline** — I execute the tasks in this session via snowball:executing-plans, batching with checkpoints for your review.
- **Not yet** — Stop here. Spec and plan are committed; you'll review and kick off execution later.

## Decision Outcome

Chose **Subagent-Driven**. Fresh subagent per task with two-stage (spec + quality) review between tasks, on the feat/argdown-plugin branch. Recommended — the launcher bootstrap and the release.yml change each get a real review gate.
