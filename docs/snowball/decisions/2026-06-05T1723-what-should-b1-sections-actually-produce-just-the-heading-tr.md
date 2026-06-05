---
title: What should B1 Sections actually produce — just the heading tree, or also which section each block belongs to
status: accepted
date: '2026-06-05T17:23:42.758Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: e1a8fa58-cd95-43ac-b795-d57af8996abf
  source_event_id: toolu_015ZyZGvrWw4n3o4NhEeMsCf
  supersedes: null
  tags:
    - ambient
---

# What should B1 Sections actually produce — just the heading tree, or also which section each block belongs to

## Context and Problem Statement

Question category: B1 scope.

## Considered Options

- **Tree + block assignment** — Build the nested section tree AND assign every block to its containing section (blocks before the first heading → none). This is the complete, useful unit — you can answer 'what's under this heading'. Matches the reference. Larger, touches how every block relates to sections. (Recommended — a section tree with no membership isn't useful on its own.)
- **Heading tree only** — Just nest the headings into a section tree (level-based parent/child), no block membership yet. Smallest possible slice; block-to-section assignment deferred to a follow-up. Leaner, but the output can't answer 'what's in this section' until the follow-up lands.

## Decision Outcome

Chose **Tree + block assignment**. Build the nested section tree AND assign every block to its containing section (blocks before the first heading → none). This is the complete, useful unit — you can answer 'what's under this heading'. Matches the reference. Larger, touches how every block relates to sections. (Recommended — a section tree with no membership isn't useful on its own.)
