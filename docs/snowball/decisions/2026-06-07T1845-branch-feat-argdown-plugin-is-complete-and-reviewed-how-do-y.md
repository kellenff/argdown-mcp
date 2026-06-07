---
title: >-
  Branch feat/argdown-plugin is complete and reviewed. How do you want to finish? (Merging — by either path — cuts the
  public v0.1.1 release the plugin pins.)
status: accepted
date: '2026-06-07T18:45:09.563Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_017ykLg3TFRpF9DggyZ2gGBH
  supersedes: null
  tags:
    - ambient
---

# Branch feat/argdown-plugin is complete and reviewed. How do you want to finish? (Merging — by either path — cuts the public v0.1.1 release the plugin pins.)

## Context and Problem Statement

Question category: Disposition.

## Considered Options

- **Push and create a PR** — Commit the decision records, push the branch, open a PR. The PR is the review gate before merge cuts the attested v0.1.1 release. Recommended — this is the branch+PR route the work was scoped for.
- **Merge to main locally** — Commit records, merge feat/argdown-plugin into main locally and push. This bypasses the PR review gate and immediately triggers the v0.1.1 release on push — contrary to the ADR's 'route release-triggering changes through a PR' rule.
- **Keep the branch as-is** — Commit the decision records onto the branch and stop. Nothing pushed; you handle the PR/merge later.
- **Discard this work** — Delete the branch and all its commits. (Decision records are untracked and survive.)

## Decision Outcome

Chose **Push and create a PR**. Commit the decision records, push the branch, open a PR. The PR is the review gate before merge cuts the attested v0.1.1 release. Recommended — this is the branch+PR route the work was scoped for.
