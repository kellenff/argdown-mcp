---
title: >-
  Attestations are always produced in CI and SHA-256 is always enforced in the launcher. How should the launcher's
  attestation *verification* behave, given keyless verification needs `gh` or a bundled sigstore lib
status: accepted
date: '2026-06-07T17:46:04.097Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01SZFi7EaGgtCwj42Qc5GoG8
  supersedes: null
  tags:
    - ambient
---

# Attestations are always produced in CI and SHA-256 is always enforced in the launcher. How should the launcher's attestation *verification* behave, given keyless verification needs `gh` or a bundled sigstore lib

## Context and Problem Statement

Question category: Verify policy.

## Considered Options

- **gh-if-present, else SHA-256** — Launcher always enforces SHA-256 (integrity, zero-dep). If `gh` is on PATH, it ALSO runs `gh attestation verify` and hard-fails on mismatch (full keyless authenticity). If `gh` is absent, it proceeds on SHA-256+TLS and prints a one-line note that provenance wasn't cryptographically verified. Provenance is always produced/auditable; enforced for gh users; never breaks non-gh users. Recommended reconciliation.
- **Require gh (strict)** — Launcher hard-requires `gh`: if absent, it refuses to run with install instructions. Every user gets enforced keyless verification — strongest guarantee — but re-introduces a toolchain dependency (gh) and breaks 'just works' for users without it.
- **Vendor sigstore-js** — Bundle sigstore-js into the plugin so the launcher verifies the attestation in pure Node, no gh, always enforced. Strongest + no external tool, but ships a large vendored dependency tree in the plugin and adds real maintenance/size.

## Decision Outcome

Chose **gh-if-present, else SHA-256**. Launcher always enforces SHA-256 (integrity, zero-dep). If `gh` is on PATH, it ALSO runs `gh attestation verify` and hard-fails on mismatch (full keyless authenticity). If `gh` is absent, it proceeds on SHA-256+TLS and prints a one-line note that provenance wasn't cryptographically verified. Provenance is always produced/auditable; enforced for gh users; never breaks non-gh users. Recommended reconciliation.
