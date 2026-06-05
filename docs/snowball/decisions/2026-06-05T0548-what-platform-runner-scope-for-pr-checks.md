---
title: What platform/runner scope for PR checks
status: accepted
date: '2026-06-05T05:48:30.420Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: c6d6d1c0-4521-4566-81c9-1253590e1bb5
  source_event_id: toolu_01YFeevwpENkk7KCqeo6PEE4
  supersedes: null
  tags:
    - ambient
---

# What platform/runner scope for PR checks

## Context and Problem Statement

Question category: Scope.

## Considered Options

- **Ubuntu only** — Fast, cheap PR feedback on ubuntu-latest. Cross-platform coverage already happens natively at release time. (Recommended.)
- **Full 4-target matrix** — Run the checks on all four release targets (linux/mac-arm/mac-intel/windows) for every PR. Thorough but slow and burns more runner minutes.
- **Ubuntu + Windows** — Linux + Windows on PRs (the two most divergent for path/line-ending bugs), skip the macOS runners for speed.

## Decision Outcome

Chose **Full 4-target matrix**. Run the checks on all four release targets (linux/mac-arm/mac-intel/windows) for every PR. Thorough but slow and burns more runner minutes.
