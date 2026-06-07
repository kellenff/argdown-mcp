# MCP Extensions — Design

- **Date:** 2026-06-07
- **Status:** Approved
- **Scope:** Extend the argdown-mcp server beyond the v1.0 trio (`parse`,
  `export_model`, `dung_extensions`) with Dung semantics tools (v1.0) and QBAF
  quantitative evaluation (v1.1 fast follow). Informed by `extensions-research.md`
  and a Gemini 3.5 Flash + MiniMax M3 chorus debate
  (`.brainstorm/chorus-20260607T125005.json`).

## Context

The MCP server ships three tools over the Layer B pipeline. Dung computation
today is limited to the **grounded extension** under a **sharp AF projection**
(argument→argument `Attack` edges only), exposed as `dung_extensions`.

The research document identifies fifteen derivable patterns (P1–P15) and four
gaps. Patterns P7–P8 and P12 (equivalence classes, bipolar support/undercut,
PCS identity) are already available via `export_model`. The load-bearing gap for
agents is **G1**: computational dispute resolution beyond grounded semantics
(P1–P6, P3–P5). **P14 (QBAF)** is scoped as a v1.1 fast follow per operator
decision.

This repo is the external glue the research doc calls for — upstream
`@argdown/core` v2 is conservative (G4) and will not close these gaps soon.

## Locked decisions

| Decision | Choice |
| --- | --- |
| Tool surface (v1.0) | 5 tools: `parse`, `export_model`, `inspect_af`, `extensions`, `accepts` |
| `dung_extensions` | Deprecated alias → `extensions` with `semantics: "grounded"`; remove in **v2** |
| Dung projection | Sharp (reference-faithful); bipolar stays in Layer B / `export_model` |
| Bipolar Dung projection | Out of scope (v1.0 and v1.1 Dung tools) |
| Well-foundedness | Metadata on `inspect_af` / `export_model`, not a standalone tool |
| `export_yaml` | `format` param on `export_model` (`json` \| `yaml`), not a separate tool |
| QBAF timing | v1.1 fast follow (after v1.0 Dung semantics ship) |
| QBAF weight source | **C:** argument metadata base + relation metadata override |
| QBAF semantics | DF-QuAD (iterative degree propagation) |
| OWL/RDF (G2) | Out of scope |
| Recursive attacks (P15) | Out of scope |
| Write-back / mutation | Out of scope |

## Research pattern mapping

| Pattern | v1.0 | v1.1 |
| --- | --- | --- |
| P1–P4 Defense, conflict-freeness, lattice, 3-valued labelling | `extensions`, `accepts` | — |
| P5 Well-foundedness collapse | `inspect_af` metadata (`is_acyclic`, SCCs) | — |
| P6 Credulous / skeptical | `accepts` with `mode` param | — |
| P7–P8, P12 Equivalence, bipolar, PCS | `export_model` (existing) | — |
| P9 Strict-mode logic bridge | Parser/model (no MCP tool) | — |
| P10–P11 k-partite, tree shape | `inspect_af` SCC metadata | — |
| P13 Scheme classification | Deferred (Phase 2 query layer) | — |
| P14 Quantitative QBAF | — | `qbaf_evaluate` |
| P15 Recursive attacks | Out of scope | Out of scope |

## Architecture

```
crates/argdown-model/src/
├── dung.rs          (existing: dung_framework, grounded_extension)
├── dung/            (new module tree, or extend dung.rs)
│   ├── af.rs        projection (extract from dung.rs)
│   ├── scc.rs       Tarjan + acyclic check + isolated-args metadata
│   ├── propagate.rs shared IN/OUT propagation primitive
│   ├── grounded.rs  O(V+E) fixed-point solver
│   ├── search.rs    SCC-decomposition + backtracking on cyclic components
│   └── semantics.rs preferred / stable / complete filters + dispatch
└── qbaf.rs          (v1.1: bipolar projection + DF-QuAD solver)

crates/argdown-tools/src/
└── lib.rs           pure functions: inspect_af, extensions, accepts, qbaf_evaluate

crates/argdown-mcp/src/
└── server.rs        rmcp adapters only
```

**Boundary discipline:** solver logic lives in `argdown-model`; tool I/O types
and source→result orchestration in `argdown-tools`; `server.rs` adapts to rmcp
only (same pattern as v1.0).

**Solver architecture (v1.0, chorus-settled):**

Two entry points sharing a propagation helper — **not** one unified engine:

- `solve_grounded(af)` — deterministic fixed-point, O(V+E), no backtracking.
- `solve_semantics(af, target)` — SCC decomposition; backtrack only on cyclic
  components; acyclic inputs short-circuit to grounded.

Stable search uses fail-fast pruning: abort branches that force UNDEC **and**
branches that violate reinstatement (maintain a defeated-set during search).

**Algorithm provenance** returned by `extensions`:

| Value | When |
| --- | --- |
| `grounded_fixpoint` | Grounded requested, or cyclic AF grounded path |
| `scc_propagation_only` | Acyclic AF, non-grounded semantics |
| `scc_with_backtracking` | Cyclic AF, preferred/stable/complete |
| `filtered_complete` | Stable via filter over complete labellings (budget fallback) |

Estimated size: ~1,400 lines solver + tool plumbing; ~600 lines fixtures.

---

## v1.0 — Phase 0 (ship first)

Serialization, projection extraction, and debuggability before expanding
semantics.

### `export_model` — add format param

Existing tool; add optional `format: "json" | "yaml"` (default `json`). Reuses
`argdown_tools::model_export` with `Format::Yaml` (already in CLI).

### `inspect_af` (new)

Exposes the projected Dung AF derived from the Layer B model.

**Input:** `{ source: String }` (inline only, consistent with existing tools).

**Output:**

```json
{
  "arguments": [{ "id": 0, "title": "A" }],
  "attacks": [{ "attacker": 1, "target": 0 }],
  "metadata": {
    "argument_count": 2,
    "attack_count": 1,
    "is_acyclic": true,
    "has_self_attacks": false,
    "strongly_connected_components": [[0], [1]],
    "isolated_arguments": []
  }
}
```

Projection rules unchanged from B6b: argument→argument `Attack` edges only;
supports, undercuts, statement-level attacks excluded.

`isolated_arguments` is **metadata only** — solver may use as propagation seed
but must not bypass correctness checks.

### `dung_extensions` → deprecated alias

Register `extensions` as the canonical tool. Keep `dung_extensions` as a
deprecated alias that calls `extensions` with `semantics: "grounded"` and
identical response shape to today's `DungResult` (IN/OUT/UNDEC partition).

Document removal in v2. Update plugin skill and `/argdown:analyze` command to
prefer `extensions`.

---

## v1.0 — Phase 1 (semantics engine)

### `extensions` (new, canonical)

**Input:**

```json
{
  "source": "...",
  "semantics": "grounded | preferred | stable | complete"
}
```

Default: `"preferred"` (agents asking "which arguments survive?" most often
want preferred over grounded).

**Output:**

```json
{
  "semantics": "preferred",
  "algorithm": "scc_with_backtracking",
  "labellings": [
    { "0": "in", "1": "out" }
  ],
  "extension_sets": [[0]]
}
```

- `labellings` — complete 3-valued labelings (primary shape).
- `extension_sets` — IN sets derived from each labelling (convenience).

Arguments referenced by arena id + title (same `ArgRef` pattern as today).

### `accepts` (new)

Point query: is a specific argument accepted under credulous or skeptical
reasoning?

**Input:**

```json
{
  "source": "...",
  "argument_id": 0,
  "semantics": "preferred",
  "mode": "credulous | skeptical"
}
```

**Output:**

```json
{
  "accepted": true,
  "status": "in",
  "unanimous": true,
  "witness": {
    "type": "accepted_uncontroversial",
    "labelling": { "0": "in", "1": "out" }
  }
}
```

**Witness types:**

| Type | Meaning |
| --- | --- |
| `accepted_uncontroversial` | IN in all extensions under requested semantics |
| `attacked_by_accepted` | OUT because an IN attacker exists |
| `unsupported_cycle` | UNDEC, cycle with no external defense |
| `multiple_interpretations` | IN in some extensions, not all (skeptical reject) |
| `skeptically_rejected` | Credulously IN, not skeptically |
| `undefined_argument` | `argument_id` not in AF |

**Schema docs must clarify** `accepted` vs `status` per mode:

- Credulous: `accepted = true` iff IN in ≥1 extension.
- Skeptical: `accepted = true` iff IN in all extensions.
- `status: "varies"` when credulous true but skeptical false.

Structured witness only — no natural-language generation in the crate.

### Testing

Fixtures crate (or module) with ~30 known-answer cases:

- Per semantics: grounded, preferred, stable, complete
- Edge cases: empty AF, self-attack, 2-cycle, 3-cycle (no stable), chain,
  bipartite attack, duplicate edges, anonymous arguments
- Labelling ↔ extension-set duality checks
- `accepts` witness type coverage
- Cross-check grounded against `@argdown/core` reference on shared samples
  (project convention)

Gate: `cargo build`, `test`, `fmt --check`, `clippy --all-targets`.

---

## v1.1 — QBAF fast follow

Quantitative Bipolar Argumentation Framework evaluation. Uses the **bipolar**
Layer B graph (attacks + supports + undercuts-as-attacks), not the sharp Dung
projection.

### Prerequisites

1. **Weight extraction helper** reading `canonical_metadata` on arguments.
2. **Edge metadata** — today `Edge` has no metadata field. v1.1 must either:
   - add `canonical_metadata: Option<Value>` to `Edge` during model build
     (from parser relation metadata), or
   - document that relation-level weights are unavailable until relation
     metadata is wired through Layer B.
   Target: wire relation metadata through so option C is fully supported.

### Weight resolution (locked: option C)

```
base_degree(arg)  ← metadata.weight on argument, else 0.5
edge_weight(edge) ← metadata.weight on relation, else base_degree(source_arg)
```

- Argument `{weight: 0.8}` sets the node's base degree.
- Relation `{weight: 0.3}` on a support/attack edge overrides for that edge.
- Undercuts map to attack edges in the QBAF projection (standard bipolar defeat).
- Invalid/non-numeric weights → tool error with diagnostic (do not silently default).

### `project_qbaf(&Model) -> QbafFramework`

- Nodes: all arguments (same as Dung AF).
- Attack edges: `Attack` + `Undercut` (argument-level and as resolved by B5).
- Support edges: `Support` (argument-level).
- Each edge carries resolved `edge_weight`.
- Each node carries resolved `base_degree`.

### Solver: DF-QuAD

Iterative degree propagation until fixpoint (standard QBAF semantics).
Document reference: Baroni et al. / emergent QBAF literature.

### `qbaf_evaluate` (new MCP tool)

**Input:**

```json
{
  "source": "...",
  "semantics": "df_quad",
  "threshold": 0.5
}
```

Default threshold: `0.5`. Default semantics: `"df_quad"` (only variant in v1.1).

**Output:**

```json
{
  "semantics": "df_quad",
  "threshold": 0.5,
  "degrees": [
    {
      "id": 0,
      "title": "A",
      "base": 0.8,
      "final": 0.62,
      "status": "accepted"
    }
  ]
}
```

`status`: `"accepted"` if `final ≥ threshold`, `"rejected"` if below,
`"undec"` if fixpoint leaves degree at boundary ambiguity (document exact rule
in implementation plan).

CLI: `argdown qbaf` subcommand mirroring the tool.

### v1.1 out of scope

- QBAF with recursive attacks
- Extension-ranking semantics (Skiba et al.)
- Multiple QBAF semantics variants beyond DF-QuAD
- OWL export of QBAF degrees

---

## Phase 2 (conditional, post-v1.1)

Only after v1.0 fixtures are stable:

- **Scheme/tag query tools** (P13) — if Argdown sources carry Walton scheme
  annotations and Layer B preserves them
- **Denormalized agent-optimized model projection** — separate spec (prior
  chorus on `export_model`; not blocking this work)
- **OWL/RDF export** (G2) — separate crate, not MCP tool

---

## Error handling

Unchanged from v1.0 MCP design:

- Parse failure → `parse` returns diagnostic; other tools return
  `invalid_params` with `{ offset }`.
- Serialization failure → `internal_error`.
- Invalid weight / unknown semantics enum → `invalid_params` with message.
- Solvers are total on valid AFs; no `Result` from model functions.

No `catch_unwind` in v1.0/v1.1 (fuzzed pipeline trusted).

---

## Plugin / skill updates

**v1.0 release:**

- Update `argdown-analysis` skill: document `inspect_af`, `extensions`,
  `accepts`; note `dung_extensions` deprecated.
- Update `/argdown:analyze` command routing.

**v1.1 release:**

- Add `qbaf_evaluate` to skill and command docs.

---

## Success criteria

**v1.0:**

- MCP `list_tools` returns `parse`, `export_model`, `inspect_af`, `extensions`,
  `accepts`, plus deprecated `dung_extensions`.
- `extensions` with `semantics: "grounded"` matches current `dung_extensions`
  and `@argdown/core` reference on shared samples.
- `extensions` with `preferred` / `complete` / `stable` pass fixture suite.
- `inspect_af` metadata matches manual SCC analysis on test graphs.
- `export_model` with `format: "yaml"` round-trips via existing `from_yaml`.

**v1.1:**

- `qbaf_evaluate` returns expected degrees on fixture graphs with known DF-QuAD
  answers.
- Weight resolution C verified: argument base, relation override, defaults.

---

## Decisions considered

- **One-tool-per-semantics vs parametric `extensions`.** Chose parametric
  (chorus unanimous). Avoids schema bloat; P6 credulous/skeptical belongs in
  `accepts`, not separate tools.
- **Bipolar Dung projection.** Rejected. Agents read support/undercut via
  `export_model`; Dung layer stays sharp. QBAF v1.1 is the correct home for
  bipolar + numeric semantics.
- **`well_foundedness_check` tool.** Rejected. Informational value to agents
  is low; `is_acyclic` metadata suffices.
- **`conflict_sets` tool.** Rejected. Query `inspect_af` attacks instead.
- **QBAF in v1.0.** Deferred to v1.1 per operator; keeps v1.0 scope focused
  on closing G1 with reference-faithful Dung semantics first.

## Chorus reference

Full transcript: `.brainstorm/chorus-20260607T125005.json`

Cast: Gemini 3.5 Flash (`gemini-synth`) + MiniMax M3 (`pragmatist`), 3 rounds,
lightweight Argdown critic (round 1 unavailable; rounds 2–3 OK).

Surviving consensus: ~5-tool surface, SCC-decomposition solver, two entry points
(grounded vs search), structured witnesses, sharp projection, honest ~1,400-line
solver budget.
