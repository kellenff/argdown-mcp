# Layer B — Arguments (B4a) — Design

- **Date:** 2026-06-06
- **Status:** Approved
- **Scope:** The first half of Layer B slice B4. B4a turns the flat
  `Block::Argument` AST into an **equivalence-class model** — a registry of
  unique argument entities (one per `<Title>` that appears as a top-level
  block) plus a block→entity assignment, computed in a new
  `argdown_model::arguments` module from `&argdown_core::Document`. The
  function is pure and **total**; strictness ("first definition wins; later
  definitions are conflicts") is surfaced as data, not as a `Result`.
  Representation-only: unit-tested in isolation; `argdown-mcp` remains a
  placeholder.

## Context

B4 (argument model + resolved PCS roles/inference) was **split** into B4a and
B4b, mirroring the A2a/A2b and A5a/A5b precedent: B4a is the argument
equivalence-class model — a near-direct mirror of B3 (statements) — while B4b
is the genuinely new, cross-cutting PCS-role / inference / argument↔PCS-linkage
work, which goes through an M2 brain-jam before its own spec.

The parser produces two kinds of `Argument`:

- **Titled definition** — `<A>: The argument.`
  `Argument.title == "A"`, `Argument.is_reference == false`,
  `Argument.description == "The argument."`.
- **Titled reference** — `<A>` (no body) or `<A>{meta}`.
  `Argument.title == "A"`, `Argument.is_reference == true`,
  `Argument.description == ""`.

Unlike `Statement`, an `Argument` is **always titled** (`title: String`, not
`Option<String>`), so B4a has no "untitled, skip it" case — and therefore no
analog to B3's `plain_text_statement_is_not_an_entity` test.

Every block sharing a title refers to **one** underlying argument. Multiple
`<A>`-titled blocks form an **equivalence class** — the canonical argument, the
references to it, and the "redefined" cases. B4a makes that class explicit,
gives each a stable `ArgumentId`, and surfaces redefinitions as data so B4b
(PCS↔argument linkage) and B5 (relations) can resolve any titled-argument block
to its entity.

| Slice | Produces | Depends on |
| ----- | -------- | ---------- |
| B1 Sections | nested section tree + block→section assignment | — |
| B2 Metadata/YAML | `parse_metadata` over `&Metadata` → `Value` | — |
| B3 Statements | `build_statements` → `Statements` | B2 |
| **B4a Arguments** | **`build_arguments` → `Arguments` (registry + block→entity + conflicts)** | **B2** |
| B4b PCS roles | resolved PCS roles/inference + argument↔PCS linkage | B3, B4a |
| B5 Relations | resolved, deduped dialectical edges between nodes | B3, B4 |
| B6 Tags / map | tag registry; node+edge map (the `dung` consumer) | B2–B5 |

B4a has the same dependency on **B2** that B3 has: `canonical_metadata` is the
parsed YAML of the first definition's metadata block, via `parse_metadata`.
B2's `MetadataError` is consumed internally; B4a is total and absorbs it as
`canonical_metadata: None`.

## Decisions

B4a inherits B3's three representation calls verbatim (B3's spec named itself
"the template B4 will follow"); no new representation question arises, so no
brain-jam:

1. **Identity + canonical content.** `Argument { id, title,
   canonical_description: Option<String>, canonical_metadata: Option<Value> }`
   — the B3 shape with `text` renamed `description`.
2. **First definition wins; later definitions are strict conflicts**, surfaced
   as data on `Arguments::conflicts` (an `ArgumentConflict` per redefined
   title), not as a `Result`.
3. **`canonical_description: Option<String>`** — `None` means "referenced but
   never defined"; `Some("")` (defined as empty) is a distinct, rare case.

Also settled, all mirroring B3:

- **An entity exists for every titled argument, even references-only.** Created
  on first occurrence (definition or reference — whichever comes first).
- **Top-level `Block::Argument` only.** Arguments appearing *only* as
  `RelationTarget::Argument` or as `InlineKind::ArgumentMention` are not
  entities here — exactly as B3 excludes relation-target and mention
  statements. Whether those should seed entities is a B4b/B5 linkage question.
- **No new external dependency.** Reuses B2's `parse_metadata` / `Value` and
  `std::collections::HashMap`.
- **One function, total, B3-parallel.** `build_arguments(&Document) ->
  Arguments` is the entire public surface.
- **No `Model` aggregate type yet.** B3 deferred the first aggregate to "B4 if
  needed"; if one is warranted it belongs to B4b (which composes arguments,
  statements, and resolved PCS), not B4a.

## Architecture

A new module `crates/argdown-model/src/arguments.rs` in the existing
`argdown-model` crate. Picked up by the `members = ["crates/*"]` glob. **No new
dependency.** `argdown-mcp` is not modified.

Public surface — one pure, total function plus four types:

```rust
use argdown_core::{Block, Document, Span};
pub use crate::metadata::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgumentId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    pub id: ArgumentId,
    pub title: String,
    pub canonical_description: Option<String>,
    pub canonical_metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentConflict {
    pub title: String,
    pub canonical_span: Span,
    pub conflicting_spans: Vec<Span>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Arguments {
    pub arguments: Vec<Argument>,
    pub block_arguments: Vec<Option<ArgumentId>>,
    pub conflicts: Vec<ArgumentConflict>,
}

pub fn build_arguments(document: &Document) -> Arguments
```

## Algorithm — single pass with a title→id map

Identical in structure to `build_statements`, swapping `Block::Statement` →
`Block::Argument`, `s.text` → `a.description`, and `canonical_text` →
`canonical_description`. Because `Argument.title` is a `String` (not
`Option`), the entity is resolved unconditionally for every `Block::Argument`
(no `.title.as_ref().map(...)`):

1. For each `Block::Argument`, get-or-create the entity by title in a
   `HashMap<String, ArgumentId>` (created on first occurrence).
2. For a **definition** (`!is_reference`): if `canonical_description.is_none()`,
   set it from `a.description`, set `canonical_metadata` via
   `a.metadata.as_ref().map(parse_metadata).transpose().ok().flatten()`, and
   record the canonical span in a `HashMap<String, Span>`; otherwise this is a
   redefinition — push `a.span` to the per-title `ArgumentConflict`
   (`canonical_span` looked up from the recorded map).
3. Push the resolved `Option<ArgumentId>` to `block_arguments`; every
   non-`Argument` block pushes `None`.
4. Drain the per-title conflict map and sort by first-appearance source order.

Invariants (all matching B3): entity created on first occurrence;
`canonical_description` is `None` until the first definition; a second
definition does not replace the canonical (it adds a conflicting span);
non-argument blocks push `None`; B2 errors absorbed.

## Error handling

None. `build_arguments` is total. Strictness is data (`conflicts`). B2's
`parse_metadata` is the only partial function called, and its error is absorbed
as `canonical_metadata: None` — the B3 model/validator seam.

## Testing (TDD)

Failing-test-first per behavior, in the new module, gated by `cargo test`,
`cargo clippy -D warnings`, `cargo fmt`. Twelve tests (B3's thirteen minus the
plain-text case, which has no argument analog):

1. **Empty document** → `Arguments::default()`.
2. **Single definition** (`<A>: desc`) → 1 entity, `canonical_description =
   Some("desc")`, `block_arguments = [Some(0)]`, no conflicts.
3. **Single reference** (`<A>`) → 1 entity, `canonical_description = None`.
4. **Definition + later reference** → 1 entity, both blocks map to it.
5. **Reference + later definition** → 1 entity, canonical filled by the
   definition.
6. **Redefinition** (`<A>: d1`, `<A>: d2`) → canonical `d1`, 1 conflict, 1
   conflicting span.
7. **Three distinct titles** → 3 entities in source order.
8. **Non-argument blocks** (heading, statement, PCS) → all `None`, 0 entities,
   `block_arguments` index-aligned with `document.blocks`.
9. **Three redefinitions** → 1 conflict with 2 conflicting spans.
10. **Canonical metadata parsed** (`<A>: desc {key: value}`) →
    `canonical_metadata = Some(Value::Mapping{...})` containing `key`.
11. **Title normalization** — parser trims (`< A >: desc` → title `A`); B4a
    doesn't re-normalize.
12. **Conflicts sorted in source order** of title first appearance.

Tests use `argdown-parser` (dev-dependency) to build `Document` inputs from
real Argdown (B1/B2/B3-parallel pattern).

## Out of scope (YAGNI; noted for later slices)

- **PCS roles / inference / argument↔PCS linkage** — that is B4b (brain-jam
  first).
- **A `Model` aggregate type** — B4b if warranted.
- **Inline / relation-target arguments as entities** — consumers find them via
  the AST; B4a handles top-level argument blocks only.
- **"Referenced but never defined" detection** — validator concern; B4a
  surfaces the data (`canonical_description: None`).
- **Typed accessors / query methods on `Arguments`** — the model is data;
  consumers walk it.

## Summary

B4a is the argument half of B4: it turns titled `Block::Argument` instances
into an equivalence-class model with a strict redefinition rule surfaced as
data. One function, no new dependency, one new module, B3-parallel structure.
B2's `parse_metadata` is consumed internally for `canonical_metadata`; B2
errors are absorbed. PCS roles and argument↔PCS linkage are deferred to B4b.
