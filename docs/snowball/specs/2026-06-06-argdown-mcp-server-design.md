# Argdown MCP Server — Design

- **Date:** 2026-06-06
- **Status:** Approved
- **Scope:** Turn the `argdown-mcp` placeholder binary into a real MCP server
  (`rmcp` over stdio) exposing `parse` / `export_model` / `dung_extensions` over
  the existing parser + Layer B pipeline.

## Context

The parser (Layer A) and the full semantic model (Layer B, B1–B6b) are complete,
and this repo recently gained JSON/YAML serialization of the Layer B `Model`
(`argdown_model::to_json` / `to_yaml`) plus a validating import. The only piece
of the original "D. MCP server" plan still missing is the protocol layer itself:
`crates/argdown-mcp/src/main.rs` is a placeholder that parses an empty string and
prints. This design wires the existing pipeline behind MCP tools so agents — the
sole intended consumers — can parse and analyze Argdown.

**Strategic context (a paused decision this design must survive):** there is an
open, not-yet-resolved decision to possibly replace the Argdown input format with
YAML entirely (deferred pending a chorus debate; reference compatibility was
already judged low-value). A Gemini + MiniMax chorus debate on the export payload
(below) concluded the **resolved `Model` is the format-independent artifact**: if
input moves to YAML, an AST-shaped export would rot, but the resolved Model
survives because it is downstream semantics, not parse output. Building this
server now is therefore largely no-regret — only the `parse` tool is coupled to
the Argdown surface syntax.

## Locked decisions

| Decision | Choice |
| --- | --- |
| SDK | `rmcp` (official Rust MCP SDK), features `server, macros, transport-io, schemars` |
| Transport | stdio, on a tokio **current-thread** runtime (single I/O-bound connection) |
| Tools | `parse`, `export_model`, `dung_extensions` (the reference trio, one renamed) |
| `export_json` → `export_model` | The payload is the resolved Model, so the tool is named after its content, not the serialization format (the reference's `export_json` is a misnomer for a Model payload) |
| Input | inline only — every tool takes `{ source: String }`; no filesystem access |

## Architecture & crate layout

`argdown-mcp` gains a dependency on `argdown-model` (today it depends only on
`argdown-core` + `argdown-parser`), since it builds the Model.

```
crates/argdown-mcp/src/
├── main.rs     thin: #[tokio::main(flavor = "current_thread")]; build the server,
│               serve over stdio(), await shutdown.
├── server.rs   the rmcp boundary — ArgdownServer + #[tool_router(server_handler)]
│               with the three #[tool] methods + ServerHandler (name/version/
│               instructions). Each #[tool] method only adapts a pure handler
│               result into rmcp Json / text Content / ErrorData.
└── tools.rs    PURE core — functions over &str → plain data, no rmcp types:
                  summarize(&str) -> ParseResult
                  model_json(&str) -> Result<String, ToolError>
                  dung(&str)       -> Result<DungResult, Diagnostic>
                + local I/O types (SourceInput, ParseResult, ParseSummary,
                  DungResult, ArgRef, Diagnostic, ToolError).
```

**Boundary discipline** (messy edges, pure core): all domain logic lives in
`tools.rs` as pure functions reusing `argdown_parser::parse`,
`argdown_model::build_model` / `to_json` / `dung_framework` /
`grounded_extension`. `server.rs` is the only module that touches rmcp and stays
trivial — so the logic is unit-testable without standing up the protocol, and the
glue has nothing to test but adaptation.

Workspace deps added: `rmcp`, `tokio`, `schemars` (`serde` / `serde_json` already
present).

## Tool I/O contracts

All three tools take `SourceInput { source: String }` (derives `Deserialize` +
`schemars::JsonSchema`; lives in `tools.rs`).

### `parse`
*"Parse Argdown source; returns a syntax summary, or a diagnostic with byte
offset."* Returns structured `Json<ParseResult>`. A syntax error is a normal
result — never a protocol error.

```
ParseResult  { ok: bool, summary: Option<ParseSummary>, diagnostic: Option<Diagnostic> }
ParseSummary { blocks, headings, statements, arguments, relations, pcs: usize,
               has_frontmatter: bool }
Diagnostic   { message: String, offset: usize }
```

`summary` counts are AST block-kind counts (syntactic). The resolved/semantic
view is `export_model`'s job.

### `export_model`
*"Returns the resolved Layer B model (statements, arguments, PCS roles,
dialectical edges, conflicts) as JSON — not the raw AST or source."* Pipeline:
`parse → build_model → to_json`. Returns the pretty-printed **Model JSON as text
content** (reuses `argdown_model::to_json`).

### `dung_extensions`
*"Compute the grounded extension under Dung's abstract argumentation framework;
returns IN/OUT/UNDEC arguments."* Pipeline: `parse → build_model →
dung_framework → grounded_extension`, mapping each `ArgumentId` to its title via
the Model's argument arena. Returns structured `Json<DungResult>`:

```
DungResult { in: [ArgRef], out: [ArgRef], undec: [ArgRef] }  // field `in_`, serde-renamed "in"
ArgRef     { id: usize, title: Option<String> }
```

**Structured vs text output (deliberate):** `parse` and `dung_extensions` return
structured `Json<…>` over small local types worth an output schema.
`export_model` returns **text JSON** to avoid a `schemars::JsonSchema` derive
sweep across the entire Model type closure in `argdown-core` + `argdown-model`;
the Model already has a serializer (`to_json`), and the reference's `export_json`
likewise returns JSON. Structured `Json<Model>` is a documented future
enhancement (requires adding `JsonSchema` to the Model closure).

## Error handling

The pipeline has exactly one domain failure source, which keeps this simple:
`build_model`, `dung_framework`, and `grounded_extension` are total (issues are
data, never `Result`).

- **Parse failure** — `parse` returns `ok:false` + `Diagnostic{message, offset}`;
  `export_model` / `dung_extensions` cannot fulfill their contract, so they return
  an rmcp `invalid_params` error with the message and `{ offset }` in the error
  `data`.
- **Serialization failure** (rare) — `to_json` fails only if parsed metadata holds
  a non-string mapping key. `model_json` returns `Result<String, ToolError>` where
  `ToolError = Parse(Diagnostic) | Serialize(String)`; the server maps `Serialize`
  → rmcp `internal_error`. `dung`'s fn returns `Result<DungResult, Diagnostic>`
  (only parse can fail).
- **Bad input** (missing/mistyped `source`) — rmcp rejects at deserialization; no
  code of ours.
- **Empty source** — parses to an empty document; zero counts / empty model /
  empty partitions. No special-casing.
- **No `catch_unwind`** — the parser + Layer B are fuzzed and total, so v1 trusts
  them; panic isolation is a noted future hardening option.

`ServerHandler` advertises name `argdown-mcp`, version from `CARGO_PKG_VERSION`,
and instructions describing the three tools and "prefer inline `source`."

## Testing & verification

- **Unit tests (pure `tools.rs`, no runtime):** summary counts on a multi-block
  doc + `has_frontmatter`; malformed source → `ok:false` with in-range `offset`;
  `model_json` reparsed to assert top-level keys + PCS item count (mirrors the
  existing export tests); `dung` on a known attack doc → expected IN/OUT (reuse
  the B6b spec's `{b IN, a OUT}` example); parse-failure path returns
  `ToolError::Parse` with offset.
- **In-process integration smoke test:** stand up `ArgdownServer` over an
  in-memory rmcp duplex transport, connect a client, assert `list_tools` returns
  the three, call each, assert shapes. Degrades to documented manual verification
  if the in-process harness proves fiddly — it does not block.
- **End-to-end:** `cargo run -p argdown-mcp`, connect via MCP Inspector
  (`npx @modelcontextprotocol/inspector`), call all three on a sample; cross-check
  `dung_extensions` against the live `@argdown/core` reference MCP on the same doc
  (the project's "track the reference" convention).
- **Gate:** `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy
  --all-targets`.

## Decisions considered

- **`export_model` payload — A (Model) vs B (our AST) vs C (reference-exact AST).**
  Chose **A**. A Gemini + MiniMax chorus debate reached decisive consensus: the
  Model is the format-independent downstream artifact (survives a future YAML
  migration, where B and C would rot together); B is a hidden-cost compromise
  delivering neither A's utility nor C's interop; C is dead weight against a moving
  target and contradicts the already-made "shed reference compat" call.
  Source-quoting needs (the Model drops surface text) are served by the separate
  `parse` tool, not by polluting the export. (Critic's Dung scoring was
  unavailable due to a `jsr:@argdown/cli` fetch failure; its structured
  steelman/anti-steelman survived. Caveat noted: Model "format-independence" is
  strong but not absolute — PCS roles/conflicts are defined against Argdown
  semantics.)
- **Tool name — keep `export_json` vs rename.** Chose **`export_model`**: name the
  content, not the serialization; a one-token, pre-launch change that removes a
  real misnomer now that the payload is the Model and reference parity is no longer
  a goal.
- **Pruned as out-of-scope.** The debate drifted (rounds 2–3) into agent-optimized
  *denormalized* schemas and a *write-back/mutation* toolset (patch_source,
  optimistic-concurrency `model_hash`, closed mutation primitives). These
  presuppose file input and a write surface — both excluded by the inline-only,
  read-only reference-trio scope. Usefully, this pressure-test *confirms* the
  scoping is coherent (no file to patch ⇒ no concurrency hole). A denormalized
  read projection + `schema_version` are recorded as possible future read-side
  enhancements; v1 ships the plain Model dump already built.

## Out of scope (YAGNI / future)

- `export_yaml` and an import/load tool (the import path is the piece most tied to
  the paused YAML decision).
- File-path input; HTTP/SSE transport; structured `Json<Model>` output;
  denormalized agent-optimized projection; `schema_version` wrapper; write-back /
  mutation tools; panic isolation.

## Blast-radius (design gate, report-only)

Graph backend (repo is indexed). Change scope **high** (~14 files via graph
expansion through the workspace manifest), failure-impact **high** (fan-out ~347),
action-risk **low** (purely additive). The decomposition flag fired but is an
artifact of touching `Cargo.toml`; the server is one cohesive subsystem and is not
split. Payload choice (A/B/C) does not materially change blast radius — it was
decided on design merits.

## Success criteria

- `cargo run -p argdown-mcp` starts an MCP server over stdio that an MCP client can
  connect to and `list_tools` returns `parse`, `export_model`, `dung_extensions`.
- Each tool returns the contracted output on valid input and the contracted
  diagnostic/error (with byte offset) on a parse failure.
- `dung_extensions` matches the `@argdown/core` reference on a shared sample.
- `cargo build` / `test` / `fmt --check` / `clippy --all-targets` are clean.
