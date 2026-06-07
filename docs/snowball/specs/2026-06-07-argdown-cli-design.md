# Argdown CLI — Design

- **Date:** 2026-06-07
- **Status:** Approved
- **Scope:** Add a command-line binary `argdown` with feature parity to the MCP
  server (`parse` / `export` / `dung`). Reads source from **stdin**, writes
  results to **stdout**, diagnostics to **stderr**, and signals errors with
  **non-zero exit codes**. Completes ports-and-adapters by extracting the pure
  tool core into a shared crate both the MCP server and the CLI depend on.

## Context

The MCP server (`crates/argdown-mcp`) already exposes the parser + Layer B
pipeline as three tools over a thin rmcp boundary. Its domain logic lives in a
**pure core** — `crates/argdown-mcp/src/tools.rs`, functions over `&str → plain
data` with no protocol types — while `server.rs` only adapts those results onto
JSON-RPC. A CLI is the same shape: a second adapter that maps the *same three
functions* onto Unix conventions (stdin/stdout/stderr/exit codes) instead of
onto JSON-RPC.

The only friction is that the pure core currently lives *inside* the
`argdown-mcp` crate, so a separate CLI crate cannot reach it without either
dragging in rmcp/tokio or extracting it. This design extracts it.

**YAML continuity.** The MCP server design (2026-06-06) listed `export_yaml` as
out of scope, "tied to the paused YAML decision." The subsequent MADR
(2026-06-07, *"Where should YAML/JSON export be exposed?"*) resolved that export
belongs in the Rust library API as the foundation (`argdown_model::to_json` /
`to_yaml`, both already implemented). The CLI is the first **surface** to expose
that latent YAML export, via `export --format json|yaml`. The MCP `export_model`
tool stays JSON-only and is unchanged.

## Locked decisions

| Decision | Choice |
| --- | --- |
| Crate layout | Approach A — extract `crates/argdown-tools` (pure core); add `crates/argdown-cli` (binary). Both adapters depend on `argdown-tools`; neither depends on the other. |
| Error model | **Unix-native** — any parse failure → diagnostic (message + byte offset) on stderr + non-zero exit, across all subcommands. Success → result on stdout, exit 0. `parse` doubles as a validator. |
| Output formats | `export` gains `--format json\|yaml` (default `json`), surfacing the latent `to_yaml`. `parse` / `dung` emit JSON. |
| Arg parsing | `clap` (derive), confined to the `argdown-cli` leaf crate. Auto `--help` / `--version` / usage errors. |
| Input | **stdin only** — no positional file argument. Matches the MCP's inline-only direction and avoids the file-I/O permission concern that deprecated the MCP `path` input. |
| Subcommand names | `parse` / `export` / `dung` (idiomatic short forms; the MCP tools are `parse` / `export_model` / `dung_extensions`). |

## Architecture & crate layout

```
crates/
  argdown-core/      unchanged — AST / Block
  argdown-parser/    unchanged — parse(&str) -> Result<Document, _>
  argdown-model/     unchanged — build_model, to_json, to_yaml, dung_framework, grounded_extension
  argdown-tools/     NEW   — pure orchestration core (relocated from argdown-mcp/src/tools.rs)
  argdown-mcp/       SLIM  — SourceInput + server.rs; depends on argdown-tools (feature "schemars")
  argdown-cli/       NEW   — clap binary `argdown`; depends on argdown-tools
```

`argdown-tools` depends on `argdown-core` + `argdown-parser` + `argdown-model`
(it orchestrates `parse → build_model → {to_json|to_yaml|dung}`, a layer above
both parser and model — which is exactly why this logic could not live in either
of those crates and lived in the mcp crate until now). `members = ["crates/*"]`
already covers the two new crates; no workspace-member edit needed.

**Boundary discipline (messy edges, pure core).** All domain logic stays in
`argdown-tools` as pure `&str → data` functions. `server.rs` (rmcp) and
`argdown-cli/main.rs` (clap + stdio) are the only modules that touch a protocol
or the process environment, and each stays trivial — adaptation only.

## Shared core API (`argdown-tools`)

The functions move from `argdown-mcp/src/tools.rs` essentially as-is, with one
addition to support format selection:

```rust
pub fn summarize(source: &str) -> ParseResult                        // unchanged
pub fn dung(source: &str) -> Result<DungResult, Diagnostic>          // unchanged

pub enum Format { Json, Yaml }
pub fn model_export(source: &str, format: Format) -> Result<String, ToolError>
//   subsumes the old model_json:
//   parse(source) -> build_model -> match format { Json => to_json, Yaml => to_yaml }
//   parse failure  -> ToolError::Parse(Diagnostic)
//   serialize fail -> ToolError::Serialize(String)
```

Result types move with the functions: `Diagnostic`, `ParseSummary`,
`ParseResult`, `ArgRef`, `DungResult`, `ToolError`. They keep `Serialize`; the
`schemars::JsonSchema` derive moves **behind a default-off `schemars` feature**,
since deriving an output schema is an MCP adapter concern, not a core one. The
MCP crate enables `argdown-tools/schemars`; the CLI does not.

`Format` stays clap-free. The CLI owns a `#[derive(clap::ValueEnum)]
OutputFormat { Json, Yaml }` and maps it onto `argdown_tools::Format` with a
small `From` impl — keeping clap out of the shared core.

**MCP crate changes (mechanical):**

- `Cargo.toml`: keep the protocol deps (rmcp/tokio/schemars/serde); add
  `argdown-tools = { workspace = true, features = ["schemars"] }`. The shared
  logic leaves via the source files below, not the manifest.
- `server.rs`: `use argdown_tools::{...}`; `export_model` calls
  `model_export(&source, Format::Json)`. `SourceInput` (the rmcp input struct,
  `Deserialize + JsonSchema`) stays in the mcp crate — it is an MCP concern the
  CLI never uses.
- `lib.rs`: remove `pub mod tools`.
- `main.rs`: update the module doc comment.
- The existing in-process integration test is untouched and still passes
  (behavior is unchanged).

The graph confirms `tools` is referenced only inside `argdown-mcp`, so the
relocation is single-crate-local.

## CLI surface & I/O contract (`argdown-cli`)

Binary `argdown`, three subcommands mapping 1:1 to the MCP tools. Each reads the
**entire stdin** as the Argdown source.

| Command | stdin | stdout on success | core fn |
| --- | --- | --- | --- |
| `argdown parse` | source | `ParseSummary` as JSON | `summarize` |
| `argdown export [-f, --format json\|yaml]` | source | model as JSON (default) or YAML | `model_export` |
| `argdown dung` | source | `DungResult` (`in` / `out` / `undec`) as JSON | `dung` |

- `parse` doubles as a validator: valid input → summary JSON on stdout, exit 0;
  malformed input → nothing on stdout, diagnostic on stderr, non-zero exit.
- Empty stdin is valid (parses to an empty document → zero counts).
- `--format` is accepted only by `export`. Default `json`.
- clap supplies `--help` / `--version` and rejects unknown flags/subcommands.

`parse` / `dung` serialize their result structs to JSON via `serde_json`
(pretty). `export` prints the `String` `model_export` returns verbatim.

## Error handling & exit codes

| Outcome | stdout | stderr | exit |
| --- | --- | --- | --- |
| success | result (JSON / YAML) | — | `0` |
| parse failure (any subcommand) | — | `argdown: <message> (at byte <offset>)` | `1` |
| serialize failure (`export` only) | — | `argdown: <message>` | `1` |
| bad CLI usage (unknown flag/subcommand) | — | clap usage message | `2` (clap default) |

- stderr is human-readable (the Unix-native choice). The byte `offset` from the
  `Diagnostic` is preserved in the message so editors/scripts can locate the
  fault.
- Serialize failure (a non-string metadata mapping key — the only `to_json` /
  `to_yaml` failure source) has no offset, so its message omits the byte clause.
- No `--error-format json` — YAGNI until requested.

## Testing & verification

- **`argdown-tools` unit tests:** the existing `tools.rs` tests relocate here
  unchanged (summary counts + `has_frontmatter`; malformed → in-range offset;
  `dung` on the known `{B IN, A OUT}` attack). Added: `model_export(Json)` is
  byte-equal to the old `model_json` output (contract test guarding the
  refactor), and `model_export(Yaml)` round-trips back to an equal model (reuses
  the existing `argdown_model` YAML round-trip helpers).
- **`argdown-cli` integration tests** (`tests/cli.rs`): drive the real binary via
  `env!("CARGO_BIN_EXE_argdown")` + `std::process::Command`, piping stdin and
  asserting on stdout / stderr / exit status. **Zero new dev-dependencies** (no
  `assert_cmd`), honoring lean-deps. Cases: each subcommand happy path; `parse`
  on malformed input → empty stdout, non-empty stderr, exit 1; `export --format
  yaml` produces YAML; `export` default is JSON; an unknown subcommand exits 2.
- **Gate:** `cargo test --workspace --locked`, `cargo fmt --check`, `cargo
  clippy --all-targets -D warnings` — all extend to the new crates automatically.

## Decisions considered

- **Crate placement — A (extract `argdown-tools`) vs B (second `[[bin]]` in
  `argdown-mcp`) vs C (`argdown-cli` depends on `argdown-mcp`).** Chose **A**.
  It completes the ports-and-adapters split already implied by `tools.rs` being
  pure: one core, two thin adapters, neither pulling the other's protocol deps.
  B keeps the smallest diff but makes a crate named "mcp" ship a non-MCP CLI that
  drags rmcp/tokio, violating one-clear-purpose; keeping it honest needs
  feature-gates that erase the small-diff benefit. C is strictly worse than A on
  deps — the CLI would transitively compile rmcp/tokio to reach pure functions,
  with the dependency direction backwards (CLI → server). The cost of A is a
  contained, graph-verified single-crate relocation plus moving the `JsonSchema`
  derive behind a feature.
- **Error model — Unix-native vs byte-parity vs hybrid.** Chose **Unix-native**.
  The MCP `parse` folds a parse failure into a normal `{ok:false}` payload
  (never a protocol error); a faithful CLI instead maps failure onto stderr +
  non-zero exit so `argdown parse` works as a validator in shell pipelines.
  Parity here is **capability parity, not byte parity** — the three operations
  are identical, only the failure channel differs. Byte-parity (always exit 0 for
  `parse`, emit `{ok:false}` to stdout) was rejected as un-idiomatic; the hybrid
  (structured stdout + stderr + non-zero exit) was rejected as emitting on two
  streams for no consumer that asked for it.
- **Output formats — JSON-only vs `--format json|yaml`.** Chose **`--format`**.
  `to_yaml` already exists beside `to_json`; surfacing it costs one match arm and
  makes the CLI the home for the YAML surface the 2026-06-07 MADR was weighing.
  Strict JSON-only parity was the YAGNI-safe default but leaves a built capability
  unexposed at the only surface that naturally hosts it.
- **Arg parsing — clap vs hand-rolled vs lexopt.** Chose **clap (derive)**.
  Lean-deps is about keeping the *recognizer* (parser/model) minimal; clap lands
  only in the leaf binary crate and buys robust `--help`, usage errors, and
  cheap extensibility. Hand-rolled saves the dep but re-implements help/usage by
  hand and ages poorly as flags grow; lexopt splits the difference without
  clearing clap's ergonomics.

## Out of scope (YAGNI / future)

- File-path / positional-argument input (stdin only).
- Machine-readable stderr (`--error-format json`), color, quiet/verbose flags.
- `--format yaml` on `parse` / `dung` (only `export` has a model worth two
  serializations; the summary and Dung partition are JSON-only).
- Exposing YAML at the MCP layer (`export_model` stays JSON-only).
- A `load`/import subcommand, or any write-back / mutation surface.

## Blast-radius (design gate)

Skipped the formal graph report (`explicitSkip`): the change is dominated by
**additive** new files (two new crates) with a single contained refactor — the
graph confirms `tools` is referenced only within `argdown-mcp` (server methods +
doc comment), so the relocation's fan-out is single-crate-local and
compile-checked by the existing CI gate. Touching the two Cargo manifests is the
only cross-crate edit. If a graph-backed impact report is wanted before
implementation, run `snowball:blast-radius` with the `design` preset over the
projected paths.

## Success criteria

- `argdown parse < doc.argdown` prints a `ParseSummary` JSON on valid input
  (exit 0) and a stderr diagnostic with byte offset + non-zero exit on malformed
  input.
- `argdown export < doc.argdown` defaults to JSON; `argdown export --format yaml`
  emits YAML; both match `argdown_model::to_json` / `to_yaml` output.
- `argdown dung < doc.argdown` prints the IN/OUT/UNDEC partition as JSON.
- The MCP server is behavior-unchanged: its integration test still lists and
  calls `parse` / `export_model` / `dung_extensions`.
- `cargo test --workspace --locked` / `cargo fmt --check` / `cargo clippy
  --all-targets -D warnings` are clean.
