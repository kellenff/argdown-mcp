---
title: >-
  A derived Deserialize parses a Model structurally but can't guarantee its cross-reference invariants hold (ids in
  range, edge endpoints valid, inference->conclusion indices valid). How should from_json/from_yaml handle that
status: accepted
date: '2026-06-07T05:17:53.419Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 13a4ad24-8404-4895-9fa2-c1301e9dc8af
  source_event_id: toolu_01V2RPPrSNej6nsiFwSmWoBK
  supersedes: null
  tags:
    - ambient
---

# A derived Deserialize parses a Model structurally but can't guarantee its cross-reference invariants hold (ids in range, edge endpoints valid, inference->conclusion indices valid). How should from_json/from_yaml handle that

## Context and Problem Statement

Question category: Validation posture.

## Considered Options

- **Validate at boundary** — Deserialize structurally, then run an intra-Model invariant check; return Err on any violation so a returned Model is sound. Round-trips the exported Model JSON and honors parse-don't-validate. (Checks what's verifiable from the Model alone; ~+50 lines + tests.)
- **Plain symmetric** — Just derive Deserialize; from_json/from_yaml return Result on structural parse only. Smallest change, mirrors the export exactly. Caller owns soundness — a forged/hand-edited file can yield a Model that panics downstream (e.g. out-of-bounds in dung_framework).
- **Rebuild via build_model** — Deserialize the Layer A Document instead, then re-run build_model to mint a guaranteed-correct Model. Soundest by construction, but round-trips Document JSON — NOT the Model JSON you currently export (would also need Document export). A different feature, really.

## Decision Outcome

Chose **Validate at boundary**. Deserialize structurally, then run an intra-Model invariant check; return Err on any violation so a returned Model is sound. Round-trips the exported Model JSON and honors parse-don't-validate. (Checks what's verifiable from the Model alone; ~+50 lines + tests.)
