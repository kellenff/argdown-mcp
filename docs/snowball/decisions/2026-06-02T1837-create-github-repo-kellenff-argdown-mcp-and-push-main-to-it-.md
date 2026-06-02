---
title: Create GitHub repo kellenff/argdown-mcp and push main to it. What visibility
status: accepted
date: '2026-06-02T18:37:39.324Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: f04cd5f5-ce00-43a9-940b-bd4c5d0fc7c5
  source_event_id: toolu_01HSaj2P4FKSWwG4LPaBUVJi
  supersedes: null
  tags:
    - ambient
---

# Create GitHub repo kellenff/argdown-mcp and push main to it. What visibility

## Context and Problem Statement

Question category: Repo visibility.

## Considered Options

- **Private** — gh repo create kellenff/argdown-mcp --private --source=. --remote=origin --push. Only you (and invited collaborators) can see it.
- **Public** — gh repo create kellenff/argdown-mcp --public --source=. --remote=origin --push. Anyone can see the code and the committed decision records.

## Decision Outcome

Chose **use git@github.com:kellenff/argdown-mcp.git**. 
