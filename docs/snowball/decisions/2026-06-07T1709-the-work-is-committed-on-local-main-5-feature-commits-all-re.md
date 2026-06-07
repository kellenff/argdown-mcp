---
title: >-
  The work is committed on local `main` (5 feature commits, all reviewed & green). `origin/main` has diverged by 1
  commit, so pushing requires reconciling first. How do you want to finish
status: accepted
date: '2026-06-07T17:09:29.761Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_0163itcp5imiXWLb6XJuerv7
  supersedes: null
  tags:
    - ambient
---

# The work is committed on local `main` (5 feature commits, all reviewed & green). `origin/main` has diverged by 1 commit, so pushing requires reconciling first. How do you want to finish

## Context and Problem Statement

Question category: Disposition.

## Considered Options

- **Commit records, keep local** — Commit the snowball decision records onto main, then stop. Nothing pushed. You handle the origin/main divergence and push yourself when ready. Safest given the remote has a commit we don't have. Recommended.
- **Commit records, pull --rebase + push** — Commit decision records, then `git pull --rebase origin main` to absorb the 1 divergent remote commit, re-run the test gate, and `git push`. I'll stop and surface any rebase conflict rather than force anything.
- **Keep as-is, don't commit records** — Leave everything exactly as it is, including the uncommitted decision records. No commits, no push.

## Decision Outcome

Chose **Commit records, pull --rebase + push**. Commit decision records, then `git pull --rebase origin main` to absorb the 1 divergent remote commit, re-run the test gate, and `git push`. I'll stop and surface any rebase conflict rather than force anything.
