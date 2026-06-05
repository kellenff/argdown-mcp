# Layer B Metadata (B2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use snowball:subagent-driven-development (recommended) or snowball:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the second Layer-B slice — a new `argdown_model::metadata` module whose `parse_metadata(&Metadata) -> Result<Value, MetadataError>` turns the raw YAML content the parser already captures into a parsed YAML value tree, using `noyalib` (the maintained YAML 1.2 library) through its `compat-serde-yaml` feature for a drop-in `serde_yaml` 0.9 surface.

**Architecture:** A new module `crates/argdown-model/src/metadata.rs` in the existing `argdown-model` crate (B1's home). Picked up by the existing `members = ["crates/*"]` workspace glob. New dependency: `noyalib` with the `compat-serde-yaml` feature (and its transitive `serde`). The public surface is one pure function plus one error type plus a re-export of the `Value` type — the same B1-parallel pattern (focused module, public re-exports, `argdown-mcp` untouched). Pure and partial — `Result`-returning, unlike B1's total `build_sections`, because the raw content can be invalid YAML.

**Tech Stack:** Rust (edition 2024, stable toolchain; noyalib 0.0.7 requires rustc 1.85+ — workspace is already on edition 2024 so this is satisfied). New external dep: `noyalib = { version = "0.0.7", features = ["compat-serde-yaml"] }` (pure Rust, no FFI, no unsafe, YAML 1.2 strict). Tests use `argdown-parser` (dev-dependency) to build `Document` inputs from real Argdown (same pattern as B1's sections tests).

**Spec:** `docs/snowball/specs/2026-06-05-layer-b-metadata-design.md`

**Branch:** Commit directly to `main` — consistent with the project convention; B2 is purely additive (a new module in an existing crate; the parser/core/MCP stays untouched) and does not bump the workspace version, so the version-gated release workflow will not fire.

---

## File Structure

| File | Responsibility | Change |
| ---- | -------------- | ------ |
| `Cargo.toml` (root) | Workspace dependency table | Modify: add `noyalib` path dep |
| `Cargo.lock` | Locked dependency graph | Modify: regenerated when the crate adds the dep |
| `crates/argdown-model/Cargo.toml` | New dep on `noyalib` | Modify: add `noyalib` to `[dependencies]` |
| `crates/argdown-model/src/metadata.rs` | `MetadataError` type, `parse_metadata` function, tests | Create |
| `crates/argdown-model/src/lib.rs` | Crate root: module decl + public re-exports | Modify: add `mod metadata; pub use metadata::{...};` |

---

### Task 1: Add `noyalib` dependency and scaffold `metadata.rs`

**Files:**
- Modify: `Cargo.toml` (root) — `[workspace.dependencies]`
- Modify: `crates/argdown-model/Cargo.toml` — `[dependencies]`
- Create: `crates/argdown-model/src/metadata.rs`
- Modify: `crates/argdown-model/src/lib.rs` — module decl + re-exports

- [ ] **Step 1: Add `noyalib` to the workspace dependency table**

In the root `Cargo.toml`, add `noyalib` under `[workspace.dependencies]` (leave the existing path deps and `winnow`/`criterion` as they are):

```toml
noyalib = { version = "0.0.7", features = ["compat-serde-yaml"] }
```

- [ ] **Step 2: Add `noyalib` to the model crate's dependencies**

In `crates/argdown-model/Cargo.toml`, add `noyalib` under `[dependencies]` (the workspace-table line is the only new entry; leave the existing `argdown-core` line and the `[dev-dependencies]` block as they are):

```toml
[dependencies]
argdown-core = { workspace = true }
noyalib = { workspace = true }
```

- [ ] **Step 3: Create `metadata.rs` with the types and a stub `parse_metadata`**

Create `crates/argdown-model/src/metadata.rs`:

```rust
//! Metadata parsing (Layer B, slice B2).
//!
//! Turns the verbatim raw YAML content the parser already captures in
//! [`argdown_core::Metadata`] into a parsed value tree. Pure and partial —
//! [`parse_metadata`] returns `Result` because the raw content can be invalid
//! YAML. Both element metadata (`{…}`) and document frontmatter
//! (`===…===`) flow through this function; the parser produces the same
//! [`argdown_core::Metadata`] shape for both, so B2 does not distinguish them.

use argdown_core::Metadata;

pub use noyalib::compat::serde_yaml::Value;

/// A metadata parse failure. Carries a human-readable message and the byte
/// offset within the raw content where parsing failed (so callers can point
/// at the failing token in the source).
#[derive(Debug)]
pub struct MetadataError {
    pub message: String,
    pub offset: usize,
}

/// Parse the raw YAML content of a `Metadata` into a `Value` tree.
///
/// Accepts any YAML root: mapping, sequence, scalar, null, or tagged value.
/// Element metadata and document frontmatter both flow through this
/// function — the parser produces the same `Metadata { raw, span }` shape
/// for both, so B2 does not distinguish them.
pub fn parse_metadata(_meta: &Metadata) -> Result<Value, MetadataError> {
    Err(MetadataError {
        message: "parse_metadata: not yet implemented".to_string(),
        offset: 0,
    })
}
```

- [ ] **Step 4: Wire the module into `lib.rs`**

In `crates/argdown-model/src/lib.rs`, add the module declaration and re-exports. The current `lib.rs` is:

```rust
//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly.

mod sections;

pub use sections::{Section, SectionId, Sections, build_sections};
```

Replace it with (adding `mod metadata;` and a re-export line — leave the `sections` block and its comment untouched):

```rust
//! Semantic model for Argdown documents (Layer B).
//!
//! Assembles the flat AST produced by `argdown-parser` into higher-level
//! structure. Grows by slice; B1 provides section assembly, B2 provides
//! metadata parsing.

mod metadata;
mod sections;

pub use metadata::{MetadataError, Value, parse_metadata};
pub use sections::{Section, SectionId, Sections, build_sections};
```

- [ ] **Step 5: Build and run the full CI gate**

Run: `cargo fmt --all`
Then: `cargo clippy --workspace --all-targets --locked -- -D warnings`
Then: `cargo build --workspace --locked`
Then: `cargo test --workspace --locked`

Expected:
- `cargo fmt` reformats nothing (or only the new files, no diff).
- `cargo clippy` and `cargo build` succeed; the model crate compiles against `noyalib` (resolving its dependency tree and writing to `Cargo.lock`).
- `cargo test` shows the previous test counts unchanged (3 core + 120 parser + 9 model = 132 passing). `_meta` is intentionally unused in the stub; the leading underscore keeps clippy quiet.

If `cargo build` fails on `noyalib` MSRV, check `rustc --version` (must be ≥ 1.85). If it fails on a missing `std` feature in the compat shim, change the workspace-dep line to `noyalib = { version = "0.0.7", default-features = false, features = ["std", "compat-serde-yaml"] }` and re-run.

- [ ] **Step 6: Commit (including `Cargo.lock`)**

```bash
git add Cargo.toml Cargo.lock crates/argdown-model
git commit -m "feat: scaffold argdown-model::metadata with noyalib dep (B2)"
```

---

### Task 2: Implement `parse_metadata` (TDD)

**Files:**
- Modify: `crates/argdown-model/src/metadata.rs`

- [ ] **Step 1: Add a failing test for the happy path (mapping root)**

In `crates/argdown-model/src/metadata.rs`, add a `#[cfg(test)] mod tests` block at the end of the file. Use the function-stub identifier `_meta` (currently unused) as the test's input — the test will panic at the `.unwrap()` against the stub, which is the expected red:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use argdown_core::{Metadata, Span};

    #[test]
    fn parses_mapping_root() {
        // B2 sees the raw content between the braces; for "{k: v}" the
        // captured Metadata.raw is the string "k: v".
        let meta = Metadata {
            raw: "k: v".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(
            matches!(v, Value::Mapping(_)),
            "expected Value::Mapping, got {v:?}"
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p argdown-model parses_mapping_root`
Expected: FAIL — the stub returns `Err(MetadataError { message: "parse_metadata: not yet implemented", .. })`, so the `.unwrap()` in the test panics. The test name appears in the failure output.

- [ ] **Step 3: Replace the stub with the one-liner implementation**

In `crates/argdown-model/src/metadata.rs`, replace the body of `parse_metadata` and remove the leading underscore from the parameter (it's now used). The doc comment above the function stays as it is. Replace the function definition with:

```rust
/// Parse the raw YAML content of a `Metadata` into a `Value` tree.
///
/// Accepts any YAML root: mapping, sequence, scalar, null, or tagged value.
/// Element metadata and document frontmatter both flow through this
/// function — the parser produces the same `Metadata { raw, span }` shape
/// for both, so B2 does not distinguish them.
pub fn parse_metadata(meta: &Metadata) -> Result<Value, MetadataError> {
    noyalib::compat::serde_yaml::from_str(&meta.raw).map_err(|e| MetadataError {
        message: e.to_string(),
        offset: e.location().map_or(0, |m| m.index()),
    })
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p argdown-model parses_mapping_root`
Expected: PASS. (The error offset is in `[0, raw.len()]`; the test doesn't pin a specific offset value, so a zero offset is acceptable on the happy path.)

- [ ] **Step 5: Format, lint, and run all crate tests**

Run: `cargo fmt --all`
Then: `cargo clippy --workspace --all-targets --locked -- -D warnings`
Then: `cargo test -p argdown-model`
Expected: `cargo fmt` makes no changes; `cargo clippy` is clean; `cargo test` reports `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (the existing 9 B1 tests plus the new `parses_mapping_root`).

- [ ] **Step 6: Commit**

```bash
git add crates/argdown-model/src/metadata.rs
git commit -m "feat: implement parse_metadata via noyalib compat (B2)"
```

---

### Task 3: Add coverage tests (scalar, sequence, nested, error cases, roundtrips)

**Files:**
- Modify: `crates/argdown-model/src/metadata.rs` — extend `mod tests`

- [ ] **Step 1: Add the 12 coverage tests**

In `crates/argdown-model/src/metadata.rs`, extend the `mod tests` block. The existing `parses_mapping_root` test stays as it is. Append the following tests inside the same `mod tests { ... }` block (after the closing `}` of `parses_mapping_root`):

```rust
    #[test]
    fn parses_scalar_string_root() {
        let meta = Metadata {
            raw: "hello".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::String(ref s) if s == "hello"));
    }

    #[test]
    fn parses_scalar_int_root() {
        let meta = Metadata {
            raw: "42".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Number(_)));
    }

    #[test]
    fn parses_scalar_bool_root() {
        let meta = Metadata {
            raw: "true".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn parses_scalar_null_root() {
        let meta = Metadata {
            raw: "null".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        assert!(matches!(v, Value::Null));
    }

    #[test]
    fn parses_sequence_root() {
        let meta = Metadata {
            raw: "[a, b, c]".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        let Value::Sequence(seq) = v else {
            panic!("expected Value::Sequence");
        };
        assert_eq!(seq.len(), 3);
    }

    #[test]
    fn parses_mapping_with_multiple_entries() {
        let meta = Metadata {
            raw: "k: v\nn: 1".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        let Value::Mapping(map) = v else {
            panic!("expected Value::Mapping");
        };
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn parses_nested_mapping() {
        let meta = Metadata {
            raw: "a:\n  b: c".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let v = parse_metadata(&meta).unwrap();
        // Outer is a mapping with one key "a".
        let Value::Mapping(map) = v else {
            panic!("expected outer Value::Mapping");
        };
        assert_eq!(map.len(), 1);
        // Inner value for "a" is itself a mapping with one key "b".
        let inner = map.into_iter().next().unwrap().1;
        let Value::Mapping(inner_map) = inner else {
            panic!("expected inner Value::Mapping");
        };
        assert_eq!(inner_map.len(), 1);
    }

    #[test]
    fn empty_raw_is_an_error() {
        // A frontmatter with no body: `===\n===`.
        let meta = Metadata {
            raw: "".to_string(),
            span: Span { start: 0, end: 0 },
        };
        assert!(parse_metadata(&meta).is_err());
    }

    #[test]
    fn invalid_yaml_is_an_error() {
        // Mismatched indentation.
        let meta = Metadata {
            raw: "a: b\n  c: d".to_string(),
            span: Span { start: 0, end: 0 },
        };
        let err = parse_metadata(&meta).unwrap_err();
        // Offset is in [0, raw.len()]. The lib may report 0 if it cannot
        // localize the failure; that's still a valid (in-range) offset.
        assert!(err.offset <= meta.raw.len());
    }

    #[test]
    fn error_offset_within_raw() {
        // For a raw with a known failure point, the offset should be inside
        // the raw (not a global document offset). We don't pin to a specific
        // byte index because the lib's exact localization is an
        // implementation detail; we only check the coordinate space.
        let raw = "good: 1\n  bad_indent: oops".to_string();
        let meta = Metadata {
            raw: raw.clone(),
            span: Span {
                start: 1000,
                end: 1000 + raw.len(),
            },
        };
        let err = parse_metadata(&meta).unwrap_err();
        assert!(
            err.offset <= raw.len(),
            "offset {} should be ≤ raw.len() {} (offset must be in the raw coordinate space, not the global document space)",
            err.offset,
            raw.len()
        );
    }

    #[test]
    fn element_metadata_roundtrip() {
        // Parse a heading with trailing `{k: v}` metadata; capture the raw
        // from the AST; parse it back and confirm it's a one-entry mapping.
        let doc = argdown_parser::parse("# Top\n{ k: v }").unwrap();
        let heading = match &doc.blocks[0] {
            argdown_core::Block::Heading(h) => h,
            other => panic!("expected Heading, got {other:?}"),
        };
        let meta = heading
            .metadata
            .as_ref()
            .expect("heading should have metadata");
        let v = parse_metadata(meta).unwrap();
        let Value::Mapping(map) = v else {
            panic!("expected Value::Mapping");
        };
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn frontmatter_roundtrip() {
        // Parse a document with leading `===…===` frontmatter containing a
        // `title: X` mapping; capture the raw from Document.frontmatter;
        // parse it back and confirm the mapping has a `title` key.
        let doc = argdown_parser::parse("===\ntitle: X\nauthor: Y\n===\n\n# Top").unwrap();
        let fm = doc.frontmatter.as_ref().expect("document should have frontmatter");
        let v = parse_metadata(fm).unwrap();
        let Value::Mapping(map) = v else {
            panic!("expected Value::Mapping");
        };
        assert!(map.contains_key("title"));
    }
```

- [ ] **Step 2: Run all crate tests**

Run: `cargo test -p argdown-model`
Expected: `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (the 9 B1 sections tests + the 1 B2 test added in Task 2 + the 12 B2 tests added in this task).

If any test fails, the test code or implementation has a bug — fix it before continuing. The implementation is one line; bugs are most likely in the test assertions (e.g., a `matches!` pattern that doesn't match a `Value` variant you expected).

- [ ] **Step 3: Commit**

```bash
git add crates/argdown-model/src/metadata.rs
git commit -m "test: cover scalar, sequence, nested, error, roundtrip (B2)"
```

---

### Task 4: Final CI gate and clean tree

**Files:**
- Modify: (no source files; the gate itself)

- [ ] **Step 1: Run the full CI gate exactly as `ci.yml` runs it**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
```

Expected:
- `fmt --check` exits 0 with no output.
- `clippy` is clean (no warnings, no errors).
- `build` succeeds.
- `test` ends with `test result: ok.` for every crate (3 core + 120 parser + 22 model = 145 total). The model crate now reports 22 tests (9 B1 sections + 1 B2 happy-path + 12 B2 coverage).

These mirror the CI `fmt` and `check` jobs, so a clean local pass predicts a green push.

- [ ] **Step 2: Confirm a clean working tree**

Run: `git status --short`
Expected: only `docs/snowball/decisions/observations.jsonl` modified (snowball-hook auto-append) and `.idea/` untracked (gitignored since the B1 button-up commit). No source-tree modifications.

- [ ] **Step 3: Commit any hook-appended observations**

```bash
git add docs/snowball/decisions/observations.jsonl
# only commit if the diff is non-empty; otherwise skip
git diff --cached --quiet || git commit -m "chore: snowball observations from B2 implementation session"
```

(If `git diff --cached --quiet` returns 0, there's nothing to commit — skip the commit and move on.)

---

## Self-Review

**Spec coverage:**
- New `argdown_model::metadata` module depending on `noyalib`, `parse_metadata(&Metadata) -> Result<Value, MetadataError>` → Task 1 (scaffold) + Task 2 (impl). ✓
- Re-export of the `Value` type from the noyalib compat shim → Task 1 Step 4. ✓
- `MetadataError { message, offset }` with `offset` sourced from `e.location().map_or(0, |m| m.index())` → Task 2 Step 3. ✓
- One function for both element metadata and frontmatter (the parser produces the same `Metadata` shape for both) → Task 3 Step 1 roundtrip tests cover both. ✓
- `noyalib` with the `compat-serde-yaml` feature, noyalib version 0.0.7 → Task 1 Steps 1–2. ✓
- 13 spec-mandated tests (8 root-type tests, 3 edge-case tests, 2 roundtrip tests) → Task 2 Step 1 (1 test) + Task 3 Step 1 (12 tests) = 13 tests. ✓
- TDD discipline: Task 2 has a failing-test-first step (Step 2 verifies failure, Step 4 verifies pass) → matches the spec's TDD note. ✓
- `argdown-mcp` not modified → Task 1 modifies only `argdown-model` and the workspace `Cargo.toml`/`Cargo.lock`; no `argdown-mcp` step. ✓
- Out-of-scope items absent: no `Model` aggregate, no typed `Metadata` view, no absolute-source-offset reporting, no per-element validation, no other YAML lib → respected. ✓

**Placeholder scan:** No "TBD", "TODO", "implement later", "fill in details", or "similar to Task N". All 13 test bodies are spelled out in Task 3 Step 1; the implementation body is spelled out in Task 2 Step 3. ✓

**Type/name consistency:** `MetadataError`, `parse_metadata`, `Value`, `Metadata`, `Span` are used identically across all tasks. The function signature `parse_metadata(meta: &Metadata) -> Result<Value, MetadataError>` is identical in Task 1 (with `_meta` and stub body) and Task 2 (with `meta` and real body). The `use` statement at the top of `metadata.rs` (`use argdown_core::Metadata;`) is consistent with the spec's algorithm block, which uses `noyalib::compat::serde_yaml::from_str(&meta.raw)`. The `pub use` in `lib.rs` re-exports `MetadataError`, `Value`, and `parse_metadata` — matching the types defined in `metadata.rs`. ✓

---

## Summary

B2 is the second slice of Layer B. The plan scaffolds the new `metadata` module and its `noyalib` dependency, implements the one-liner `parse_metadata` function via TDD, adds 12 coverage tests (mapping, scalar string/int/bool/null, sequence, multi-entry mapping, nested mapping, empty, invalid YAML, error offset, element roundtrip, frontmatter roundtrip), and closes with the same CI gate B1 used. Four tasks, 18 steps total; the implementation itself is one line; the bulk of the work is the test surface (13 tests in `argdown-model`'s new metadata module, on top of the 9 B1 sections tests). argdown-mcp is not modified.
