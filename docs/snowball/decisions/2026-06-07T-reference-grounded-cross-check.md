---
title: >-
  MCP extensions v1.0 reference cross-check — live @argdown/core Dung MCP unavailable;
  B6b canonical probe encoded in CI tests
status: accepted
date: '2026-06-07T20:00:00.000Z'
deciders:
  - kellen
snowball:
  schema_version: '1.0'
  source: operator
  confidence: high
  capture_mechanism: verification
  session_id: null
  source_event_id: null
  supersedes: null
  tags:
    - mcp-extensions
    - reference-parity
---

# MCP extensions v1.0 reference cross-check

## Context

The MCP server plan (`docs/snowball/plans/2026-06-06-argdown-mcp-server.md`, Step 5) requires
cross-checking `dung_extensions` against the live `@argdown/core` MCP on the canonical B6b sample:

```argdown
<A>: a

<B>: b
  -> <A>
```

Expected partition: **B IN**, **A OUT**, **UNDEC empty**.

## What we tried

| Path | Result |
| --- | --- |
| `jsr:@argdown/cli` | Package fetch failed (not found) |
| `npm @argdown/core@2.0.1` | Installed; no Dung / extension API exposed |
| `npm @argdown/cli@2.0.0` | No `dung` subcommand |
| Our CLI / MCP on B6b sample | `{B in, A out}` — matches expected probe |

## Decision

Accept **encoded B6b probe parity** as the reference cross-check for v1.0:

- `crates/argdown-tools/tests/reference_parity.rs` — library-level grounded partition
- `crates/argdown-mcp/tests/integration.rs::grounded_cross_check_matches_b6b_probe_via_mcp` — MCP wire test

Re-run live `@argdown/core` MCP cross-check when a published package again exposes
`dung_extensions` (or equivalent).

## Outcome

No divergence observed on the canonical sample. Live reference MCP unavailable at ship time;
CI tests prevent silent regression.
