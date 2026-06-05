# Layer B — Metadata (B2) — Design

- **Date:** 2026-06-05
- **Status:** Approved
- **Scope:** The second slice of Layer B. B2 turns the raw YAML content the
  parser already captures in `argdown_core::Metadata` into a parsed YAML value
  tree, in a new `argdown-model::metadata` module, computed from
  `&argdown_core::Metadata`. Both element metadata (`{…}`) and document
  frontmatter (`===…===`) flow through one function — the parser produces the
  same `Metadata { raw, span }` shape for both, so B2 doesn't care which kind
  it's parsing. Representation-only: unit-tested in isolation; `argdown-mcp`
  remains a placeholder.

## Context

The parser (A1–A5b) already recognizes and captures metadata blocks, both
element-level (`{key: value}` trailing an element) and document-level
(leading `===…===` frontmatter). For both, it stores the verbatim raw inner
content plus the source span in `argdown_core::Metadata { raw: String, span:
Span }` and deliberately does **not** parse the YAML — that work is deferred
to Layer B.

Layer B is decomposed into six slices (B1–B6) per the project's
"ship thin vertical slices, split when scope balloons" principle. B1
(sections) shipped as the foundational first slice — smallest, zero-dependency,
representing the rest of the model. B2 is the second slice and the **first
slice that introduces a non-trivial external dependency** (a YAML library).
Its representation choice (the value type and how errors surface) is the
template B3–B6 follow, just as B1's flat-arena choice was the template for
B2's B1-parallel module shape.

| Slice | Produces | Depends on |
| ----- | -------- | ---------- |
| B1 Sections | nested section tree + block→section assignment | — |
| **B2 Metadata/YAML** | **`parse_metadata` over `&Metadata` → `noyalib::compat::serde_yaml::Value`** | **— (B1 unrelated)** |
| B3 Statement model | statement equivalence classes | — |
| B4 Argument model + PCS roles | arguments + resolved PCS roles/inference | B3 |
| B5 Relations | resolved, deduped dialectical edges between nodes | B3, B4 |
| B6 Tags / map | tag registry; node+edge map (the `dung` consumer) | B2–B5 |

B2 has **no dependency on B1** — sections and metadata are independent axes.
B3–B6 will consume the parsed `noyalib::compat::serde_yaml::Value` (B6's tag registry and B4's
argument metadata are the obvious consumers) but that is downstream
integration, not a B2 concern.

## Decisions

Three calls drove B2's shape. They are recorded here as the template B3–B6
will follow, just as B1's flat-arena decision is the template for B2's
flat-module shape.

1. **Full YAML value tree, not a narrow key→scalar map.** B2's output is
   `noyalib::compat::serde_yaml::Value` (a recursive tree of mappings, sequences, scalars, and
   null), not a hand-rolled `BTreeMap<String, String>` or a narrower scalar
   set. Rationale: argdown metadata in the wild is already flat-mostly
   (`tags: [a, b]`, `weight: 0.8`, `cited: true`), but the few nested cases
   (argument `premises: [a, b, c]` lists, statement `sources: [{...}]`
   arrays) and the sequence cases (statement `tags: [a, b, c]`) are real and
   would force a B3+ consumer to re-parse if we returned a narrower shape.
   Paying the cost of a real YAML lib once in B2 is cheaper than paying it
   again per consumer. (Rejected: a flat string map — loses type
   information; rejected: a scalar-only set — forces a second parser for
   sequences and mappings.)

2. **Accept any YAML root, not require a mapping.** `parse_metadata` returns
   whatever YAML tree comes back: `Value::Mapping`, `Value::Sequence`,
   `Value::String`, `Value::Number`, `Value::Bool`, `Value::Null`, or
   `Value::Tagged`. Rationale: keeping both element metadata and frontmatter
   on one rule keeps the API surface small (one function, one error type) and
   gives B3–B6 maximum flexibility. The "metadata = key/value" semantic is a
   consumer concern — B3 can add a `require_mapping` accessor when it
   actually needs one. (Rejected: require a mapping at the root — fights the
   "metadata can be a list of tags" use case; rejected: per-kind
   constraints — splits the API for no current benefit.)

3. **`noyalib` (with the `compat-serde-yaml` feature), not `serde_yaml` or
   `serde_yml` (both deprecated).** The chain is `serde_yaml` (deprecated
   2024) → `serde_yml` (now also deprecated, 0.0.13) → `noyalib` (the
   current maintained option). noyalib is pure Rust, no FFI, no unsafe,
   YAML 1.2 strict. We use it through the `compat-serde-yaml` feature,
   which re-exports the `serde_yaml` 0.9 surface (`from_str`, `Value`,
   `Error::location()`) — that's the path of least change for our
   spec. `MetadataError` is a local type that wraps the upstream error,
   so the public API doesn't expose the upstream type — that means we
   can move to noyalib's native API (or another YAML lib) later without
   breaking B3–B6 consumers. (Rejected: `serde-saphyr` — typed
   deserialization only, no `Value` DOM, would force us into a typed
   struct; rejected: `yaml-rust2` — no serde wrapper, would force us
   to write the conversion ourselves; rejected: handwritten YAML —
   reinvents the wheel and creates a long-tail bug surface.)

Also settled:

- **No `Result` → `Result` representation in core.** `argdown_core::Metadata`
  stays `Metadata { raw, span }`; B2 owns the parse step in `argdown-model`.
  The core remains the syntax-tree types the parser produces (B1's
  precedent: core's `Document` is never mutated by the model).
- **No `Model` aggregate type yet.** Deferred per B1's out-of-scope list —
  the second slice settles when there's something to aggregate. B2 leaves
  B3 to introduce the first aggregate.
- **One function, not two.** `parse_metadata(&Metadata) -> Result<Value,
  MetadataError>` works for both element and frontmatter. The kind
  distinction is a parser concern, not a B2 concern.

## Architecture

A new module `crates/argdown-model/src/metadata.rs` in the existing
`argdown-model` crate (B1's home). Picked up by the existing `members =
["crates/*"]` workspace glob automatically. New dependency:
`noyalib = { version = "0.0.7", features = ["compat-serde-yaml"] }` (and
its transitive `serde`). `argdown-mcp` is **not** modified in B2.

B2's entire public surface is one pure function plus one error type plus a
re-export of the value type:

```rust
pub use noyalib::compat::serde_yaml::Value;  // the value tree, re-exported for downstream slices

#[derive(Debug)]
pub struct MetadataError {
    pub message: String,
    pub offset: usize,   // byte offset within the raw content
}

pub fn parse_metadata(meta: &Metadata) -> Result<Value, MetadataError>
```

## Data types

```rust
/// A metadata parse failure. Carries a human message and the byte offset
/// within the raw content where parsing failed (so callers can point at the
/// failing token in the source).
#[derive(Debug)]
pub struct MetadataError {
    pub message: String,
    pub offset: usize,
}

/// Parse the raw YAML content of a `Metadata` into a `noyalib::compat::serde_yaml::Value` tree.
///
/// Accepts any YAML root: mapping, sequence, scalar, null, or tagged value.
/// Element metadata and document frontmatter both flow through this
/// function — the parser produces the same `Metadata { raw, span }` shape
/// for both, so B2 does not distinguish them.
pub fn parse_metadata(meta: &Metadata) -> Result<Value, MetadataError>
```

The function is **partial** (returns `Result`) — unlike `build_sections`
which was total. Rationale: the raw content can be invalid YAML (mismatched
indentation, unterminated strings, etc.), and that's a Layer-B concern the
parser doesn't (and shouldn't) pre-validate. The parser's job is to
recognize the *shape* of a metadata block (balanced braces, fence lines,
etc.); the YAML inside is opaque to it.

## Algorithm

A one-liner:

```rust
pub fn parse_metadata(meta: &Metadata) -> Result<Value, MetadataError> {
    noyalib::compat::serde_yaml::from_str(&meta.raw).map_err(|e| MetadataError {
        message: e.to_string(),
        offset: e.location().map_or(0, |m| m.index()),
    })
}
```

`noyalib::compat::serde_yaml::Error::location()` returns an `Option<Marker>`; a `Marker` has
an `index()` (byte offset within the source that was parsed). For us, the
"source that was parsed" is `meta.raw`, so the offset is already in the
right coordinate space for error reporting. (If we wanted to surface
absolute document offsets, we'd add `meta.span.start + 1` to skip the
opening fence/brace — but B2 only commits to the in-raw offset, leaving
absolute-offset reporting to B3+ if a consumer needs it.)

That's the whole algorithm. The point of B2 is to stand on `noyalib`'s
shoulders; if B2 were more than a thin wrapper, that would be a smell.

## Error handling

`MetadataError` is local to `argdown-model`. We do **not** re-export
the upstream error — that would couple the public API to the upstream
crate, making future swaps breaking changes. The mapping captures what
callers actually need: a human message and a position. Anything more
granular (kind enum, marked line/col) is YAGNI for B2; if B3+ needs it,
`MetadataError` grows fields.

The error is constructed by mapping the upstream error → `MetadataError`.
Failures B2 surfaces are exclusively "the raw is not valid YAML." There is
no "wrong root type" error (B2 accepts any root) and no "missing key"
error (the parser already validated that the metadata block shape — `{…}`
or `===…===` — is well-formed).

## Testing (TDD)

Failing-test-first per behavior, in the new module, gated by `cargo test`,
`cargo clippy --all-targets -D warnings`, and `cargo fmt`. B2 sees the
`raw` content between the braces (e.g. for a heading with `{k: v}`,
`meta.raw` is the string `k: v`); the tests below name both the source
shape and the raw that B2 actually parses.

1. mapping root: element `{k: v}` (raw `k: v`) → `Value::Mapping` with one entry
2. scalar string root: element `{hello}` (raw `hello`) → `Value::String("hello")`
3. scalar int root: element `{42}` (raw `42`) → `Value::Number(42)`
4. scalar bool root: element `{true}` (raw `true`) → `Value::Bool(true)`
5. scalar null root: element `{null}` (raw `null`) → `Value::Null`
6. sequence root: element `{[a, b, c]}` (raw `[a, b, c]`) → `Value::Sequence` with three entries
7. mapping with multiple entries: element `{k: v\nn: 1}` (raw `k: v\nn: 1`) → `Value::Mapping` with two entries
8. nested mapping: element `{a:\n  b: c}` (raw `a:\n  b: c`) → mapping containing a mapping
9. empty raw: frontmatter `===…===\n===…===` with no body (raw `""`) → `Ok(Value::Null)` (empty input is valid YAML — the YAML 1.2 null document — not an error)
10. invalid YAML: a raw with bad indentation (e.g. `"a: b\n  c: d"`) → error; the error's `offset` is between 0 and `raw.len()`
11. element roundtrip: parse a heading with `{k: v}` metadata and confirm `parse_metadata` on the captured `Metadata` returns the expected mapping
12. frontmatter roundtrip: parse a document with `===…===` frontmatter containing `title: X\nauthor: Y` and confirm `parse_metadata` on `Document.frontmatter` returns the expected mapping
13. error offset within raw: feeding a raw with a known failure point, confirm the error's offset matches the byte index of the failing token within the raw (not a global document offset)

Tests use `argdown-parser` as dev-dependency to build `Document` inputs
from real Argdown (same pattern as B1's sections tests).

## Out of scope (YAGNI; noted for later slices)

- A `Model` aggregate type — introduced when a second slice exists, per
  B1's out-of-scope list. B3 will introduce it if needed.
- A typed `Metadata` view (e.g. `BTreeMap<String, Value>` extracted from
  the root mapping) — `noyalib::compat::serde_yaml::Value` already gives callers ergonomic
  access; we can add convenience accessors when a consumer needs them.
- Absolute-source-offset error reporting — B2 commits to in-raw offset
  only. Absolute-offset reporting can be added by adding `meta.span.start`
  to the error's `offset` when a B3+ consumer needs it.
- Per-element validation (e.g., "this argument's metadata must have
  `premises: […]`") — that's B4's job. B2 is a parser, not a validator.
- A different YAML lib (`serde_norway`, handwritten, etc.) — `noyalib`
  is the maintained drop-in for `serde_yaml`. The `MetadataError` type
  means we can swap later without breaking the public API.
- B2's role in caching or memoizing parsed metadata — B2 is a pure
  function over `&Metadata`; callers cache if they need to.

## Summary

B2 is the second slice of Layer B. It does one thing: turn the raw YAML
content the parser already captures into a `noyalib::compat::serde_yaml::Value` tree, with a
local `MetadataError` type for parse failures. One function, one new
external dependency, one new module, B1-parallel structure. Out-of-scope
items are deferred to B3–B6.
