# Argdown Cargo Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the single-package repo into a three-crate Cargo workspace (`argdown-core`, `argdown-parser`, `argdown-mcp`) that builds, tests, and runs.

**Architecture:** A virtual workspace manifest at the repo root owns shared metadata and dependency versions. `argdown-core` holds pure domain types, `argdown-parser` turns text into those types using winnow (stubbed for now), and `argdown-mcp` is a thin binary that depends on both. Dependency graph is acyclic: `core ← parser ← mcp`.

**Tech Stack:** Rust (edition 2024, resolver 3), Cargo workspaces, winnow 1.x.

---

## File Structure

| Path | Responsibility |
|------|----------------|
| `Cargo.toml` (root) | Virtual workspace manifest: members glob, `[workspace.package]` shared metadata, `[workspace.dependencies]` version source. No `[package]`. |
| `src/main.rs` (root) | **Deleted** — the hello-world binary moves into `crates/argdown-mcp`. |
| `crates/argdown-core/Cargo.toml` | Manifest for the domain-model library. No internal deps. |
| `crates/argdown-core/src/lib.rs` | `Document` domain type + `Error` type. No parsing, no I/O. |
| `crates/argdown-parser/Cargo.toml` | Manifest for the parser library. Depends on `argdown-core` + `winnow`. |
| `crates/argdown-parser/src/lib.rs` | `parse(&str) -> Result<Document, Error>`. Stub grammar for now. |
| `crates/argdown-mcp/Cargo.toml` | Manifest for the server binary. Depends on `argdown-core` + `argdown-parser`. |
| `crates/argdown-mcp/src/main.rs` | Placeholder binary: parse, then print the document. |
| `Cargo.lock` | Generated on first build; committed (only `/target` is gitignored). Pins exact dependency versions. |

**Naming note:** package `argdown-core` is imported in Rust as `argdown_core` (hyphens become underscores). Same for `argdown_parser`.

---

## Task 1: Virtual workspace manifest + `argdown-core`

**Files:**
- Modify: `Cargo.toml` (replace the entire root package manifest with a virtual workspace manifest)
- Delete: `src/main.rs`
- Create: `crates/argdown-core/Cargo.toml`
- Create: `crates/argdown-core/src/lib.rs`

- [ ] **Step 1: Replace the root `Cargo.toml` with a virtual workspace manifest**

Overwrite `Cargo.toml` with exactly:

```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"

[workspace.dependencies]
winnow = "1"
argdown-core = { path = "crates/argdown-core" }
```

(`license`/`repository` are intentionally omitted until the project is ready to publish — adding them later is a one-line change in `[workspace.package]`.)

- [ ] **Step 2: Delete the root hello-world binary**

Run: `git rm src/main.rs`
Expected: `rm 'src/main.rs'`

- [ ] **Step 3: Create the `argdown-core` manifest**

Create `crates/argdown-core/Cargo.toml`:

```toml
[package]
name = "argdown-core"
version.workspace = true
edition.workspace = true

[dependencies]
```

- [ ] **Step 4: Write the failing test**

Create `crates/argdown-core/src/lib.rs`. At this point it contains only docs + the test module — `Document` and `Error` are deliberately undefined so the crate fails to compile (RED):

```rust
//! Core domain types for Argdown documents.
//!
//! These are the precise types the parser produces and the rest of the
//! program is written against. The model will grow as the grammar is
//! implemented; for now it is a minimal placeholder.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_displays_parse_message() {
        let err = Error::Parse("unexpected token".to_string());
        assert_eq!(err.to_string(), "parse error: unexpected token");
    }

    #[test]
    fn document_default_is_constructible() {
        let _doc = Document::default();
    }
}
```

- [ ] **Step 5: Run the test to verify it fails**

Run: `cargo test -p argdown-core`
Expected: FAIL — compile errors `cannot find type 'Error' in this scope` and `cannot find type 'Document' in this scope`.

- [ ] **Step 6: Implement the minimal domain types**

Insert the following ABOVE the `#[cfg(test)]` module in `crates/argdown-core/src/lib.rs` (keep the `//!` header at the very top):

```rust
/// A parsed Argdown document.
///
/// Empty for now; the parser will populate fields as the grammar is built.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {}

/// Errors produced while turning source text into a [`Document`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The parser could not interpret the input.
    Parse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(message) => write!(f, "parse error: {message}"),
        }
    }
}

impl std::error::Error for Error {}
```

- [ ] **Step 7: Run the test to verify it passes**

Run: `cargo test -p argdown-core`
Expected: PASS — `test result: ok. 2 passed; 0 failed`.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/argdown-core
git commit -m "feat: scaffold workspace and argdown-core crate

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

(The `src/main.rs` deletion was already staged by Step 2's `git rm`, so it is included in this commit automatically.)

Expected: commit succeeds; `git status` shows `src/main.rs` removed and the new crate + `Cargo.lock` added.

---

## Task 2: `argdown-parser`

**Files:**
- Modify: `Cargo.toml` (add `argdown-parser` to `[workspace.dependencies]` so the mcp crate can reference it in Task 3)
- Create: `crates/argdown-parser/Cargo.toml`
- Create: `crates/argdown-parser/src/lib.rs`

- [ ] **Step 1: Register the parser crate in workspace dependencies**

Edit the `[workspace.dependencies]` table in the root `Cargo.toml` so it reads exactly:

```toml
[workspace.dependencies]
winnow = "1"
argdown-core = { path = "crates/argdown-core" }
argdown-parser = { path = "crates/argdown-parser" }
```

- [ ] **Step 2: Create the `argdown-parser` manifest**

Create `crates/argdown-parser/Cargo.toml`:

```toml
[package]
name = "argdown-parser"
version.workspace = true
edition.workspace = true

[dependencies]
argdown-core = { workspace = true }
winnow = { workspace = true }
```

(winnow is declared now so the grammar work can begin immediately; the stub in Step 4 does not yet use it. Rust does not warn on an unused crate dependency by default.)

- [ ] **Step 3: Write the failing test**

Create `crates/argdown-parser/src/lib.rs` with the doc header and a test that calls a not-yet-defined `parse` (RED):

```rust
//! Winnow-based parser for the Argdown format.
//!
//! Turns source text into an [`argdown_core::Document`]. The grammar is a
//! stub for now and will be implemented incrementally.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_input_yields_empty_document() {
        assert_eq!(parse(""), Ok(argdown_core::Document::default()));
    }
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test -p argdown-parser`
Expected: FAIL — compile error `cannot find function 'parse' in this scope`.

- [ ] **Step 5: Implement the stub parser**

Insert ABOVE the `#[cfg(test)]` module in `crates/argdown-parser/src/lib.rs`:

```rust
use argdown_core::{Document, Error};

/// Parse Argdown source text into a [`Document`].
///
/// Currently a stub: it accepts any input and returns an empty document.
/// The real winnow grammar will replace this body.
pub fn parse(_source: &str) -> Result<Document, Error> {
    Ok(Document::default())
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p argdown-parser`
Expected: PASS — `test result: ok. 1 passed; 0 failed`.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/argdown-parser
git commit -m "feat: add argdown-parser crate with stub parse()

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: `argdown-mcp` binary

**Files:**
- Create: `crates/argdown-mcp/Cargo.toml`
- Create: `crates/argdown-mcp/src/main.rs`

No root manifest change is needed: `argdown-core` and `argdown-parser` are already in `[workspace.dependencies]`, and the `crates/*` members glob picks up the new crate automatically.

This task has no unit test: the binary is pure wiring (no domain logic of its own), so it is verified by building and running it.

- [ ] **Step 1: Create the `argdown-mcp` manifest**

Create `crates/argdown-mcp/Cargo.toml`:

```toml
[package]
name = "argdown-mcp"
version.workspace = true
edition.workspace = true

[dependencies]
argdown-core = { workspace = true }
argdown-parser = { workspace = true }
```

- [ ] **Step 2: Create the placeholder binary**

Create `crates/argdown-mcp/src/main.rs`:

```rust
//! Argdown MCP server (placeholder binary).
//!
//! For now this just exercises the parser to prove the workspace wires up.
//! The MCP protocol layer is future work.

use argdown_core::Document;
use argdown_parser::parse;

fn main() {
    let source = "";
    match parse(source) {
        Ok(document) => report(&document),
        Err(error) => eprintln!("failed to parse: {error}"),
    }
}

fn report(document: &Document) {
    println!("parsed argdown document: {document:?}");
}
```

- [ ] **Step 3: Build to verify it compiles**

Run: `cargo build -p argdown-mcp`
Expected: PASS — `Finished` with no errors.

- [ ] **Step 4: Run to verify output**

Run: `cargo run -p argdown-mcp`
Expected stdout: `parsed argdown document: Document`

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/argdown-mcp
git commit -m "feat: add argdown-mcp placeholder binary

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Workspace-wide verification

**Files:** none created; this task formats, lints, and verifies the whole workspace.

- [ ] **Step 1: Format the workspace**

Run: `cargo fmt`
Expected: no output (files already conform; if it reformats anything, that is fine).

- [ ] **Step 2: Verify formatting is canonical**

Run: `cargo fmt --check`
Expected: no output, exit code 0.

- [ ] **Step 3: Lint with clippy, warnings as errors**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: PASS — `Finished` with no warnings or errors.

- [ ] **Step 4: Build the whole workspace**

Run: `cargo build`
Expected: PASS — all three crates compile.

- [ ] **Step 5: Test the whole workspace**

Run: `cargo test`
Expected: PASS — `argdown-core` (2 tests) and `argdown-parser` (1 test) pass; `argdown-mcp` has no tests.

- [ ] **Step 6: Run the binary from the workspace root**

Run: `cargo run -p argdown-mcp`
Expected stdout: `parsed argdown document: Document`

- [ ] **Step 7: Commit any formatting changes**

```bash
git add -A
git commit -m "chore: cargo fmt across workspace" || echo "nothing to commit"
```

Expected: either a commit for formatting changes, or `nothing to commit` if Steps 1–2 made no changes.

---

## Success criteria (from the spec)

- `cargo build` and `cargo test` succeed from the repo root. (Task 4, Steps 4–5)
- `cargo run -p argdown-mcp` runs the placeholder binary. (Task 4, Step 6)
- Each crate has a focused responsibility; the dependency graph `core ← parser ← mcp` is acyclic. (Tasks 1–3 manifests)
