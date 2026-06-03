---
title: Which PCS AST representation should A3 use
status: accepted
date: '2026-06-03T03:37:32.861Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 5228b259-3782-4175-84ed-1ea224dd5973
  source_event_id: toolu_01AiKPF1m98c8pfP5q7kFda8
  supersedes: null
  tags:
    - ambient
---

# Which PCS AST representation should A3 use

## Context and Problem Statement

Question category: PCS AST.

## Considered Options

- **Refined-A: flat form items** — Pcs { items: [Statement{number,Statement} | Inference{rules,span}] }. Parser surfaces FORM (numbered statements, inference lines + rule names) as first-class items; Layer B derives premise/intermediary/main roles positionally. Consistent with A1/A2b, no role/relationship work in the parser, no Layer-B re-scan. (M2-jam recommendation.)
- **B: role-tagged statements** — Pcs { statements: [PcsStatement{number, statement, role, inference}] }. Parser assigns premise/intermediary/main roles positionally and binds each conclusion's inference to it. More immediately usable, but the jam argues role is relational meaning that belongs in Layer B, and tagging conflates form with role.

## Decision Outcome

Chose **Refined-A: flat form items**. Pcs { items: [Statement{number,Statement} | Inference{rules,span}] }. Parser surfaces FORM (numbered statements, inference lines + rule names) as first-class items; Layer B derives premise/intermediary/main roles positionally. Consistent with A1/A2b, no role/relationship work in the parser, no Layer-B re-scan. (M2-jam recommendation.)
