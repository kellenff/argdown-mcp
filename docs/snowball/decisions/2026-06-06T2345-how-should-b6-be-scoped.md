---
title: How should B6 be scoped
status: accepted
date: '2026-06-06T23:45:56.604Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e6bfcdaa-6d51-4b27-9e10-52a5d225ed4d
  source_event_id: toolu_01HnUwmrZoUWn5o2H9zhCb6u
  supersedes: null
  tags:
    - ambient
---

# How should B6 be scoped

## Context and Problem Statement

Question category: Scope.

## Considered Options

- **Split: B6a tags, B6b dung** — B6a = tag registry (inline #tags → registry + per-entity tag lists), low risk. B6b = the Dung AF map (+ extension). Two thinner slices, matching the A2a/A2b, A5a/A5b, B4a/B4b precedent.
- **Single slice B6** — Tags + Dung map (+ extension) together in one spec→plan→build. Fewer ceremony steps; larger blast radius.

## Decision Outcome

Chose **Split: B6a tags, B6b dung**. B6a = tag registry (inline #tags → registry + per-entity tag lists), low risk. B6b = the Dung AF map (+ extension). Two thinner slices, matching the A2a/A2b, A5a/A5b, B4a/B4b precedent.
