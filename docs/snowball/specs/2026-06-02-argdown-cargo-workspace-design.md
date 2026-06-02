# Argdown Cargo Workspace — Design

- **Date:** 2026-06-02
- **Status:** Approved
- **Scope:** Convert the single-package repo into a Cargo workspace with three crates.

## Goal

Establish the workspace skeleton for a high-performance Argdown
(argdown.org) implementation aimed at agent consumption via MCP. Parsing
will use [winnow](https://docs.rs/winnow) for performance and ergonomics.
This step sets up structure and separation of concerns only — it does not
implement the grammar or the MCP protocol.

## Directory layout

```
argdown-mcp/                       workspace root (virtual manifest — no root package)
├── Cargo.toml                     [workspace]: members, shared metadata, shared deps
├── .gitignore
└── crates/
    ├── argdown-core/              lib — AST + domain model types, shared error type
    │   ├── Cargo.toml
    │   └── src/lib.rs
    ├── argdown-parser/            lib — winnow parser: &str → argdown_core::Document
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── argdown-mcp/               bin — MCP server, depends on parser + core
        ├── Cargo.toml
        └── src/main.rs
```

The existing root `argdown-mcp` package is dissolved: the root `Cargo.toml`
becomes a **virtual manifest** (no `[package]` section), and the current
hello-world binary moves to `crates/argdown-mcp/src/main.rs`.

## Crate responsibilities & dependency graph

```
argdown-core   (no internal deps; std only for now)
     ▲
argdown-parser (→ argdown-core, → winnow)
     ▲
argdown-mcp    (→ argdown-parser, → argdown-core)   [bin]
```

- **argdown-core** — the precise domain types the rest of the program is
  written against (`Document`, `Statement`, `Argument`, `Relation`,
  `Section`, plus a shared error type). No parsing, no I/O. This is the
  "parse, don't validate" landing zone where external text becomes precise
  domain values.
- **argdown-parser** — winnow-based parsing only. Consumes `&str`, produces
  `argdown_core` types or a parse error. This is where the bulk of future
  tests live (table-driven over Argdown snippets).
- **argdown-mcp** — the binary. Depends on `argdown-parser` and
  `argdown-core` **explicitly** (not via re-exports), per the
  explicit-over-implicit preference.

## Workspace configuration

- `resolver = "3"`, `edition = "2024"` (matches the existing manifest).
- `[workspace.package]` holds shared `version` / `edition` / `license` /
  `repository`; members inherit via `version.workspace = true` etc.
- `[workspace.dependencies]` holds `winnow` (pinned to the current latest
  release — exact version confirmed at implementation time) and the three
  internal crates as path deps. Members reference them as
  `argdown-core = { workspace = true }`. Single source of truth for
  versions.

## Out of scope (YAGNI for this step)

- **No MCP SDK wired in.** `argdown-mcp` is a placeholder binary that calls
  the parser and prints. Choosing `rmcp` vs. another SDK is a separate
  decision.
- **No `serde`** until the JSON-export path needs it.
- **No CLI crate, no diagnostics crate** (miette / codespan) yet.
- **Parser grammar is a stub** — `parse()` returns an empty / `todo!`
  document. The actual grammar is future work. Each library ships one
  trivial passing test so `cargo test` proves the workspace wires up.

## Decisions considered

- **Crate count:** three crates (core + parser + mcp) chosen over two
  (parser + mcp, AST in parser) or parser-only. Rationale: the shared
  domain model is likely to gain a second consumer (e.g. a CLI or JSON
  exporter), and keeping it independent of winnow keeps the model pure.
- **Layout:** `crates/` subdirectory chosen over a flat root layout — keeps
  the repo root clean and follows the common multi-crate idiom.

## Success criteria

- `cargo build` and `cargo test` succeed from the repo root.
- `cargo run -p argdown-mcp` runs the placeholder binary.
- Each crate has a focused responsibility and the dependency graph is
  acyclic as drawn above.
