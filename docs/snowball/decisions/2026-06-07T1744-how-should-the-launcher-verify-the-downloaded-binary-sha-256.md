---
title: >-
  How should the launcher verify the downloaded binary? (SHA-256 gives integrity over TLS; a signature adds publisher
  authenticity but introduces key/identity management.)
status: accepted
date: '2026-06-07T17:44:30.115Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: ask-user-question
  session_id: 267bb12c-ac5f-41dc-8896-838aa2e98b39
  source_event_id: toolu_01E2Ka7hHzsFmuXVR7JfxEnP
  supersedes: null
  tags:
    - ambient
---

# How should the launcher verify the downloaded binary? (SHA-256 gives integrity over TLS; a signature adds publisher authenticity but introduces key/identity management.)

## Context and Problem Statement

Question category: Verification.

## Considered Options

- **SHA-256 baseline (now)** — Keep SHA-256 over HTTPS for v0.1, with an honest threat-model note in the spec, and record signed provenance as planned hardening. Rationale: TLS already authenticates github.com; a long-lived key in this repo's secrets adds key-management burden but little protection against the dominant threat (account compromise). Zero new infra, zero launcher deps. Recommended for v0.1.
- **Ed25519 signature now** — Sign SHA256SUMS in release.yml with an Ed25519 key (private key in GitHub secrets, public key baked into launch.mjs); the launcher verifies it with Node's built-in crypto — still zero npm deps. Real authenticity bump vs weaker threats; cost is generating/storing/rotating the key and the same-account caveat. NOT HMAC, NOT RSA.
- **Keyless attestations** — Add GitHub Artifact Attestations (actions/attest-build-provenance) — the genuine best practice, no long-lived secret, transparency-log backed. Cost: the launcher must verify via `gh attestation verify` (adds a `gh` dependency on the user machine) or the sigstore JS lib (breaks the dependency-free launcher). Heaviest, best supply-chain posture.

## Decision Outcome

Chose **Keyless attestations**. Add GitHub Artifact Attestations (actions/attest-build-provenance) — the genuine best practice, no long-lived secret, transparency-log backed. Cost: the launcher must verify via `gh attestation verify` (adds a `gh` dependency on the user machine) or the sigstore JS lib (breaks the dependency-free launcher). Heaviest, best supply-chain posture.
