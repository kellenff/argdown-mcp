# Argdown MCP Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the `argdown-mcp` placeholder binary into a real MCP server (rmcp over stdio) exposing `parse`, `export_model`, and `dung_extensions` over the existing parser + Layer B pipeline.

**Architecture:** A pure core (`tools.rs`: `&str → plain data`, no protocol types) wrapped by a thin rmcp boundary (`server.rs`: `#[tool_router]`/`#[tool]` adapters) and a trivial `main.rs` (tokio + stdio). All domain logic is unit-testable without the protocol; the rmcp glue only adapts results.

**Tech Stack:** Rust 2024, `rmcp` (server, macros, transport-io, schemars), `tokio` (current-thread), `schemars`, `serde`/`serde_json`, and the workspace crates `argdown-parser` / `argdown-model` / `argdown-core`.

**Reference spec:** `docs/snowball/specs/2026-06-06-argdown-mcp-server-design.md`

---

## File Structure

| File | Responsibility |
| --- | --- |
| `crates/argdown-mcp/Cargo.toml` | Add `rmcp`, `tokio`, `schemars`, `serde`, `serde_json`, `argdown-model` deps |
| `crates/argdown-mcp/src/tools.rs` | **Create.** Pure functions + I/O types: `SourceInput`, `Diagnostic`, `ParseSummary`, `ParseResult`, `ArgRef`, `DungResult`, `ToolError`; `summarize`, `model_json`, `dung` |
| `crates/argdown-mcp/src/server.rs` | **Create.** `ArgdownServer` + `#[tool_router]` (the 3 tools) + `#[tool_handler] ServerHandler` |
| `crates/argdown-mcp/src/main.rs` | **Replace.** tokio entrypoint: serve over stdio, await shutdown |
| `crates/argdown-mcp/tests/integration.rs` | **Create (best-effort).** In-process client↔server smoke test |

**Boundary discipline:** only `server.rs` and `main.rs` import `rmcp`. `tools.rs` imports only the `argdown_*` crates + serde/schemars. Keep it that way.

---

## A note on `rmcp` version & import paths

`rmcp` is pre-1.0 and module paths shift between minor versions. The **macro and extractor names are stable** (`#[tool_router]`, `#[tool]`, `#[tool_handler]`, `Parameters<T>`, `Json<T>`, `ErrorData`, `ServiceExt`, `stdio()`); only their import *paths* may vary. Task 1 resolves the version; if any `use` path in Task 5–7 fails to compile, run `cargo doc -p rmcp --open` (or check docs.rs/rmcp) and adjust the path — do not change the names.

**schemars version gotcha:** the `JsonSchema` derive you use *must* be the same `schemars` major version `rmcp` depends on. Task 1 pins it by reading `cargo tree`.

---

### Task 1: Dependencies

**Files:**
- Modify: `crates/argdown-mcp/Cargo.toml`

- [ ] **Step 1: Add the workspace-internal + serde deps**

Edit `crates/argdown-mcp/Cargo.toml` so `[dependencies]` reads:

```toml
[dependencies]
argdown-core = { workspace = true }
argdown-parser = { workspace = true }
argdown-model = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

- [ ] **Step 2: Add rmcp + tokio (cargo resolves latest compatible)**

Run:
```bash
cargo add rmcp -p argdown-mcp --features server,macros,transport-io,schemars
cargo add tokio -p argdown-mcp --features rt,macros,io-std
```
Expected: both added to `crates/argdown-mcp/Cargo.toml` with a resolved version.

- [ ] **Step 3: Pin schemars to rmcp's major version**

Find the version rmcp uses:
```bash
cargo tree -p argdown-mcp -i schemars 2>/dev/null | head -3
```
Then add the matching major (substitute the observed major, e.g. `0.8`):
```bash
cargo add schemars -p argdown-mcp --no-default-features --features derive
```
If `cargo tree` shows a different schemars major than `cargo add` picked, pin it explicitly: `cargo add schemars@<major-from-tree> -p argdown-mcp --no-default-features --features derive`. The goal: exactly one `schemars` version in the tree.

- [ ] **Step 4: Verify the workspace still builds (placeholder main untouched)**

Run: `cargo build -p argdown-mcp`
Expected: PASS. Unused-dependency warnings are fine; a duplicate `schemars` error means Step 3's pin is wrong — fix before continuing.

- [ ] **Step 5: Commit**

```bash
git add crates/argdown-mcp/Cargo.toml Cargo.lock
git commit -m "build: add rmcp/tokio/schemars deps to argdown-mcp"
```

---

### Task 2: `tools.rs` — parse types + `summarize`

**Files:**
- Create: `crates/argdown-mcp/src/tools.rs`
- Modify: `crates/argdown-mcp/src/main.rs` (declare `mod tools;`)

- [ ] **Step 1: Declare the module so the bin sees it**

Add this line near the top of `crates/argdown-mcp/src/main.rs` (above `fn main`):

```rust
mod tools;
```

(The placeholder `main` is replaced in Task 6; leave it for now.)

- [ ] **Step 2: Write `tools.rs` with the parse-side types and the failing test**

Create `crates/argdown-mcp/src/tools.rs`:

```rust
//! Pure tool logic: `&str` source → plain result data. No rmcp/protocol types.

use argdown_core::Block;
use argdown_parser::parse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Inline source input shared by every tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SourceInput {
    /// The Argdown source text to analyze.
    pub source: String,
}

/// A parse failure: human-readable message + byte offset into the source.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Diagnostic {
    pub message: String,
    pub offset: usize,
}

/// Syntactic block-kind counts for a successfully parsed document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ParseSummary {
    pub blocks: usize,
    pub headings: usize,
    pub statements: usize,
    pub arguments: usize,
    pub relations: usize,
    pub pcs: usize,
    pub has_frontmatter: bool,
}

/// `parse` result: a summary on success, a diagnostic on failure. Never an error.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ParseResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ParseSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<Diagnostic>,
}

/// Parse `source` and report a syntactic summary, or a diagnostic on failure.
pub fn summarize(source: &str) -> ParseResult {
    match parse(source) {
        Ok(doc) => {
            let mut summary = ParseSummary {
                blocks: doc.blocks.len(),
                headings: 0,
                statements: 0,
                arguments: 0,
                relations: 0,
                pcs: 0,
                has_frontmatter: doc.frontmatter.is_some(),
            };
            for block in &doc.blocks {
                match block {
                    Block::Heading(_) => summary.headings += 1,
                    Block::Statement(_) => summary.statements += 1,
                    Block::Argument(_) => summary.arguments += 1,
                    Block::Relation(_) => summary.relations += 1,
                    Block::Pcs(_) => summary.pcs += 1,
                }
            }
            ParseResult { ok: true, summary: Some(summary), diagnostic: None }
        }
        Err(e) => ParseResult {
            ok: false,
            summary: None,
            diagnostic: Some(Diagnostic { message: e.message, offset: e.offset }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_counts_blocks_and_frontmatter() {
        let src = "===\ntitle: T\n===\n\n# H\n\n[S]: s\n\n<A>: a\n\n(1) P\n----\n(2) C";
        let r = summarize(src);
        assert!(r.ok);
        let s = r.summary.expect("summary present on success");
        assert!(s.has_frontmatter);
        assert_eq!(s.headings, 1);
        assert_eq!(s.statements, 1);
        assert_eq!(s.arguments, 1);
        assert_eq!(s.pcs, 1);
        assert_eq!(s.blocks, s.headings + s.statements + s.arguments + s.relations + s.pcs);
        assert!(r.diagnostic.is_none());
    }

    #[test]
    fn summarize_reports_a_diagnostic_on_malformed_source() {
        // An unterminated metadata block is a parse error.
        let r = summarize("# H {unterminated");
        assert!(!r.ok);
        assert!(r.summary.is_none());
        let d = r.diagnostic.expect("diagnostic present on failure");
        assert!(d.offset <= "# H {unterminated".len());
        assert!(!d.message.is_empty());
    }
}
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cargo test -p argdown-mcp tools::`
Expected: PASS (2 tests). The implementation ships with the test in this task because the logic is a thin fold over the parser; if `summarize` had a bug the asserts would catch it.

> If `summarize_reports_a_diagnostic_on_malformed_source` fails because `"# H {unterminated"` happens to parse, swap the input for `"[A]: x { y"` (an unterminated inline-metadata block) — any input rejected by `argdown_parser::parse` works.

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-mcp/src/tools.rs crates/argdown-mcp/src/main.rs
git commit -m "feat: parse summary tool logic (summarize)"
```

---

### Task 3: `tools.rs` — `ToolError` + `model_json`

**Files:**
- Modify: `crates/argdown-mcp/src/tools.rs`

- [ ] **Step 1: Add the imports**

At the top of `tools.rs`, extend the `argdown_model` use (add a new line):

```rust
use argdown_model::{build_model, to_json};
```

- [ ] **Step 2: Add `ToolError` and the failing test**

Append to `tools.rs` (before the `#[cfg(test)]` module), add the `ToolError` type and `model_json`:

```rust
/// Why a tool could not produce its output.
#[derive(Debug)]
pub enum ToolError {
    /// The source did not parse.
    Parse(Diagnostic),
    /// The resolved model could not be serialized (e.g. non-string metadata key).
    Serialize(String),
}

/// Parse `source`, build the Layer B model, and return it as pretty-printed JSON.
pub fn model_json(source: &str) -> Result<String, ToolError> {
    let doc = parse(source)
        .map_err(|e| ToolError::Parse(Diagnostic { message: e.message, offset: e.offset }))?;
    let model = build_model(&doc);
    to_json(&model).map_err(|e| ToolError::Serialize(e.to_string()))
}
```

Then add these tests inside the existing `#[cfg(test)] mod tests`:

```rust
    #[test]
    fn model_json_serializes_the_resolved_model() {
        let json = model_json("<A>: d\n\n(1) P1\n----\n(2) C1").expect("valid model");
        let v: serde_json::Value = serde_json::from_str(&json).expect("reparses");
        let obj = v.as_object().expect("top-level object");
        for key in ["statements", "arguments", "pcs", "edges"] {
            assert!(obj.contains_key(key), "missing key {key}");
        }
        assert_eq!(v["pcs"][0]["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn model_json_returns_parse_error_with_offset() {
        let err = model_json("[A]: x { y").unwrap_err();
        match err {
            ToolError::Parse(d) => assert!(d.offset <= "[A]: x { y".len()),
            other => panic!("expected ToolError::Parse, got {other:?}"),
        }
    }
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cargo test -p argdown-mcp tools::`
Expected: PASS (4 tests total).

> If `model_json_returns_parse_error_with_offset`'s input parses on your build, substitute any source rejected by the parser (see Task 2's note).

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-mcp/src/tools.rs
git commit -m "feat: model export tool logic (model_json)"
```

---

### Task 4: `tools.rs` — Dung types + `dung`

**Files:**
- Modify: `crates/argdown-mcp/src/tools.rs`

- [ ] **Step 1: Extend the imports**

Update the `argdown_model` use line in `tools.rs` to:

```rust
use argdown_model::{ArgumentId, build_model, dung_framework, grounded_extension, to_json};
```

- [ ] **Step 2: Add the Dung types + `dung` and the failing test**

Append to `tools.rs` (before the test module):

```rust
/// A reference to an argument by its arena id and (optional) title.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ArgRef {
    pub id: usize,
    pub title: Option<String>,
}

/// The grounded extension partition: accepted / defeated / undecided arguments.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DungResult {
    #[serde(rename = "in")]
    pub in_: Vec<ArgRef>,
    pub out: Vec<ArgRef>,
    pub undec: Vec<ArgRef>,
}

/// Parse `source`, build the model, project to a Dung AF, and return the
/// grounded extension with arguments resolved to `{id, title}`.
pub fn dung(source: &str) -> Result<DungResult, Diagnostic> {
    let doc = parse(source).map_err(|e| Diagnostic { message: e.message, offset: e.offset })?;
    let model = build_model(&doc);
    let af = dung_framework(&model);
    let labelling = grounded_extension(&af);
    let to_refs = |ids: &[ArgumentId]| -> Vec<ArgRef> {
        ids.iter()
            .map(|id| ArgRef {
                id: id.0,
                title: model.arguments.get(id.0).and_then(|a| a.title.clone()),
            })
            .collect()
    };
    Ok(DungResult {
        in_: to_refs(&labelling.in_),
        out: to_refs(&labelling.out),
        undec: to_refs(&labelling.undec),
    })
}
```

Add these tests inside `mod tests`:

```rust
    #[test]
    fn dung_partitions_a_simple_attack() {
        // <B> attacks <A>: B is unattacked (IN), A is defeated (OUT).
        let d = dung("<A>: a\n\n<B>: b\n  -> <A>").expect("valid");
        let titles =
            |refs: &[ArgRef]| refs.iter().filter_map(|a| a.title.clone()).collect::<Vec<_>>();
        assert_eq!(titles(&d.in_), vec!["B"]);
        assert_eq!(titles(&d.out), vec!["A"]);
        assert!(d.undec.is_empty());
    }

    #[test]
    fn dung_returns_parse_error_with_offset() {
        let d = dung("[A]: x { y").unwrap_err();
        assert!(d.offset <= "[A]: x { y".len());
    }
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cargo test -p argdown-mcp tools::`
Expected: PASS (6 tests total).

> The attack direction matters: `<B>: b` with `  -> <A>` means B attacks A (outbound). If IN/OUT come out reversed on your build, confirm against the live reference (`dung_extensions` on the same source) and flip the expected vectors — the *partition* is what's tested, and it must match the reference.

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-mcp/src/tools.rs
git commit -m "feat: dung extension tool logic (dung)"
```

---

### Task 5: `server.rs` — the rmcp boundary

**Files:**
- Create: `crates/argdown-mcp/src/server.rs`
- Modify: `crates/argdown-mcp/src/main.rs` (declare `mod server;`)

- [ ] **Step 1: Declare the module**

Add to `crates/argdown-mcp/src/main.rs` (next to `mod tools;`):

```rust
mod server;
```

- [ ] **Step 2: Write `server.rs`**

Create `crates/argdown-mcp/src/server.rs`:

```rust
//! The rmcp boundary: adapts pure `tools` results into MCP tool responses.

use rmcp::handler::server::tool::Parameters;
use rmcp::{ErrorData, Json, ServerHandler, tool, tool_handler, tool_router};
use serde_json::json;

use crate::tools::{self, DungResult, ParseResult, SourceInput, ToolError};

/// The Argdown MCP server. Stateless — one unit value handles every request.
#[derive(Debug, Clone)]
pub struct ArgdownServer;

#[tool_router]
impl ArgdownServer {
    #[tool(
        name = "parse",
        description = "Parse Argdown source; returns a syntactic summary, or a diagnostic with a byte offset on failure. Prefer inline `source`."
    )]
    fn parse(&self, Parameters(SourceInput { source }): Parameters<SourceInput>) -> Json<ParseResult> {
        Json(tools::summarize(&source))
    }

    #[tool(
        name = "export_model",
        description = "Returns the resolved Layer B model (statements, arguments, PCS roles, dialectical edges, conflicts) as JSON — not the raw AST or source. Prefer inline `source`."
    )]
    fn export_model(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Result<String, ErrorData> {
        match tools::model_json(&source) {
            Ok(json) => Ok(json),
            Err(ToolError::Parse(d)) => {
                Err(ErrorData::invalid_params(d.message, Some(json!({ "offset": d.offset }))))
            }
            Err(ToolError::Serialize(msg)) => Err(ErrorData::internal_error(msg, None)),
        }
    }

    #[tool(
        name = "dung_extensions",
        description = "Compute the grounded extension under Dung's abstract argumentation framework; returns IN/OUT/UNDEC arguments. Prefer inline `source`."
    )]
    fn dung_extensions(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Result<Json<DungResult>, ErrorData> {
        match tools::dung(&source) {
            Ok(result) => Ok(Json(result)),
            Err(d) => Err(ErrorData::invalid_params(d.message, Some(json!({ "offset": d.offset })))),
        }
    }
}

#[tool_handler(
    instructions = "Argdown argumentation toolchain. Tools: parse (syntactic summary/diagnostics), export_model (resolved Layer B model as JSON), dung_extensions (grounded IN/OUT/UNDEC). Prefer inline `source`."
)]
impl ServerHandler for ArgdownServer {}
```

- [ ] **Step 3: Compile-check (no runtime yet)**

Run: `cargo build -p argdown-mcp`
Expected: PASS.

> If a `use` path fails (`rmcp::Json`, `rmcp::ErrorData`, `rmcp::handler::server::tool::Parameters`, or the macro names), the names are correct but the module path moved in your rmcp version — run `cargo doc -p rmcp --open` and fix the path. Common alternates: `rmcp::model::ErrorData`, `rmcp::handler::server::wrapper::{Json, Parameters}`. If `#[tool_handler(instructions = …)]` is rejected, replace that block with a manual `get_info` (still inside `#[tool_handler] impl ServerHandler for ArgdownServer {}`):
> ```rust
> #[tool_handler]
> impl ServerHandler for ArgdownServer {
>     fn get_info(&self) -> rmcp::model::ServerInfo {
>         rmcp::model::ServerInfo {
>             capabilities: rmcp::model::ServerCapabilities::builder().enable_tools().build(),
>             instructions: Some("Argdown argumentation toolchain. Tools: parse, export_model, dung_extensions. Prefer inline `source`.".into()),
>             ..Default::default()
>         }
>     }
> }
> ```

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-mcp/src/server.rs crates/argdown-mcp/src/main.rs
git commit -m "feat: rmcp tool router + server handler (ArgdownServer)"
```

---

### Task 6: `main.rs` — serve over stdio

**Files:**
- Modify: `crates/argdown-mcp/src/main.rs` (replace the placeholder body)

- [ ] **Step 1: Replace `main.rs` entirely**

Overwrite `crates/argdown-mcp/src/main.rs` with:

```rust
//! Argdown MCP server: serves `parse` / `export_model` / `dung_extensions`
//! over stdio.

mod server;
mod tools;

use rmcp::ServiceExt;
use rmcp::transport::stdio;

use server::ArgdownServer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ArgdownServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

- [ ] **Step 2: Build**

Run: `cargo build -p argdown-mcp`
Expected: PASS.

> If `rmcp::transport::stdio` is not found, use `rmcp::transport::io::stdio` (per docs.rs). If `serve`/`waiting` are missing, ensure `use rmcp::ServiceExt;` is present (it provides `.serve()` on the handler; the running service exposes `.waiting()`).

- [ ] **Step 3: Smoke-run the binary (it should block on stdin, not exit/panic)**

Run: `echo '' | cargo run -q -p argdown-mcp`
Expected: the process starts and exits cleanly on EOF (empty stdin closes the transport). No panic, no error output. (A real session is driven by an MCP client — see Task 8.)

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-mcp/src/main.rs
git commit -m "feat: serve ArgdownServer over stdio"
```

---

### Task 7: In-process integration smoke test (best-effort)

**Files:**
- Create: `crates/argdown-mcp/tests/integration.rs`

This test exercises the full protocol path in one process. It is **best-effort**: if the rmcp client API differs in your version and it won't compile after a docs check, skip it (delete the file) and rely on Task 8's manual verification — do **not** block the plan on it.

- [ ] **Step 1: Expose the server to integration tests**

Integration tests compile against the crate's *library*, but `argdown-mcp` is a binary. Add a minimal lib target so the test can import `ArgdownServer`. In `crates/argdown-mcp/Cargo.toml`, add:

```toml
[lib]
path = "src/lib.rs"
```

Create `crates/argdown-mcp/src/lib.rs`:

```rust
//! Library surface for integration tests; the binary lives in `main.rs`.
pub mod server;
pub mod tools;
```

Then in `crates/argdown-mcp/src/main.rs`, replace the `mod server;` / `mod tools;` lines with a use of the crate's own lib:

```rust
use argdown_mcp::server::ArgdownServer;
```

(Delete the now-duplicate `mod server;` and `mod tools;` from `main.rs`.)

- [ ] **Step 2: Write the integration test**

Create `crates/argdown-mcp/tests/integration.rs`:

```rust
use argdown_mcp::server::ArgdownServer;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParam;
use serde_json::json;

#[tokio::test]
async fn lists_and_calls_the_three_tools() {
    let (client_io, server_io) = tokio::io::duplex(8192);

    // Server side.
    let server = ArgdownServer.serve(server_io).await.expect("server serves");
    tokio::spawn(async move {
        let _ = server.waiting().await;
    });

    // Client side: a bare `()` is a no-capability client handler.
    let client = ().serve(client_io).await.expect("client connects");

    // list_tools → exactly our three.
    let tools = client.list_all_tools().await.expect("list tools");
    let mut names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort();
    assert_eq!(names, vec!["dung_extensions", "export_model", "parse"]);

    // parse → ok summary.
    let parsed = client
        .call_tool(CallToolRequestParam {
            name: "parse".into(),
            arguments: json!({ "source": "<A>: a" }).as_object().cloned(),
        })
        .await
        .expect("parse call");
    assert_ne!(parsed.is_error, Some(true));

    // export_model → JSON text mentioning a top-level key.
    let exported = client
        .call_tool(CallToolRequestParam {
            name: "export_model".into(),
            arguments: json!({ "source": "<A>: a\n\n(1) P\n----\n(2) C" }).as_object().cloned(),
        })
        .await
        .expect("export call");
    assert_ne!(exported.is_error, Some(true));

    client.cancel().await.ok();
}
```

- [ ] **Step 3: Run it**

Run: `cargo test -p argdown-mcp --test integration`
Expected: PASS.

> If it does not compile against your rmcp version (the client helpers `list_all_tools` / `call_tool` / `CallToolRequestParam` / `.serve()` on `()` are the most version-sensitive surface): spend one pass reconciling names via `cargo doc -p rmcp --open`. If still stuck, **delete `tests/integration.rs`**, keep the `[lib]` split (it's harmless and useful), and proceed — Task 8 covers verification manually. Note the skip in the commit message.

- [ ] **Step 4: Commit**

```bash
git add crates/argdown-mcp/Cargo.toml crates/argdown-mcp/src/lib.rs crates/argdown-mcp/src/main.rs crates/argdown-mcp/tests/integration.rs
git commit -m "test: in-process MCP integration smoke test"
```

---

### Task 8: Final gate + manual end-to-end verification

**Files:** none (verification only)

- [ ] **Step 1: Format**

Run: `cargo fmt --all && cargo fmt --all -- --check`
Expected: clean (no diff).

- [ ] **Step 2: Clippy (whole workspace, all targets)**

Run: `cargo clippy --all-targets`
Expected: no warnings. Fix any in `argdown-mcp` before committing.

- [ ] **Step 3: Full test suite**

Run: `cargo test`
Expected: PASS, including the 6 `tools::` unit tests (and the integration test if kept).

- [ ] **Step 4: Manual end-to-end via MCP Inspector**

Run:
```bash
npx @modelcontextprotocol/inspector cargo run -q -p argdown-mcp
```
In the Inspector UI: confirm `list tools` shows `parse`, `export_model`, `dung_extensions`. Call each with `{"source": "<A>: a\n\n<B>: b\n  -> <A>"}`:
- `parse` → `ok: true` with a summary.
- `export_model` → JSON with `statements` / `arguments` / `edges`.
- `dung_extensions` → `in: [{title:"B"}]`, `out: [{title:"A"}]`, `undec: []`.

Also confirm a parse error path: call `export_model` with `{"source": "[A]: x { y"}` → an error result carrying a byte `offset` in its data.

- [ ] **Step 5: Cross-check `dung_extensions` against the reference**

Using the live `@argdown/core` MCP (`dung_extensions`, `kind: inline`) on the same `<A>: a\n\n<B>: b\n  -> <A>`, confirm the IN/OUT/UNDEC partition matches our server's (the project's "track the reference" convention). Note any divergence as a follow-up; do not silently accept it.

- [ ] **Step 6: Commit any fmt/clippy fixes**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for argdown-mcp server"
```

---

## Done criteria

- `cargo run -p argdown-mcp` serves an MCP stdio server; a client's `list_tools` returns `parse` / `export_model` / `dung_extensions`.
- Each tool returns its contracted output on valid input and a diagnostic/error with byte offset on parse failure.
- `dung_extensions` matches the `@argdown/core` reference on the shared sample.
- `cargo build` / `test` / `fmt --check` / `clippy --all-targets` are clean.
