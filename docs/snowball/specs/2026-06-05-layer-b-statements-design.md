# Layer B — Statements (B3) — Design

- **Date:** 2026-06-05
- **Status:** Approved
- **Scope:** The third slice of Layer B. B3 turns the flat `Block::Statement`
  AST into an **equivalence-class model** — a registry of unique statement
  entities (one per title that appears in the document) plus a block→entity
  assignment, computed in a new `argdown_model::statements` module from
  `&argdown_core::Document`. The function is pure and **total**; strictness
  ("first definition wins; later definitions are conflicts") is surfaced as
  data, not as a `Result`. Representation-only: unit-tested in isolation;
  `argdown-mcp` remains a placeholder.

## Context

The parser (A1–A5b) already produces three kinds of `Statement`:

- **Plain text** — `Just some claim.` `Statement.title == None`,
  `Statement.is_reference == false`.
- **Titled definition** — `[A]: The claim.`
  `Statement.title == Some("A")`, `Statement.is_reference == false`,
  `Statement.text == "The claim."`.
- **Titled reference** — `[A]` (no body) or `[A]{meta}`.
  `Statement.title == Some("A")`, `Statement.is_reference == true`,
  `Statement.text == ""`.

Real Argdown documents treat every block sharing a title as a reference to
**one** underlying claim. Multiple `[A]`-titled blocks (whether definitions
or references) form an **equivalence class** — the canonical claim, the
references to it, and the "redefined" cases where a title was given two
distinct definitions. B3 makes that class explicit, gives each class a
stable id, and surfaces the "redefined" cases as data so consumers (B4
arguments, B5 relations, B6 tags/map) can resolve any titled-statement
block to its entity.

Layer B is decomposed into six slices (B1–B6) per the project's
"ship thin vertical slices, split when scope balloons" principle. B1
(sections) and B2 (metadata) are done. B3 is the third slice and the
**first slice with cross-block identity** — sections and metadata were
per-block; statements are per-title-across-blocks. B5 (relations) will
reference B3's `StatementId`; B6 (tags/map) will compose B3 with B4 and B5
into a graph. B3's representation choice (id shape, conflict handling,
canonical-content rules) is the template the later identity-bearing slices
follow, just as B1's flat-arena choice was the template for B2's
flat-module shape and B2's `MetadataError`-wrapping pattern was the
template for shielding B3 from upstream churn.

| Slice | Produces | Depends on |
| ----- | -------- | ---------- |
| B1 Sections | nested section tree + block→section assignment | — |
| B2 Metadata/YAML | `parse_metadata` over `&Metadata` → `noyalib::compat::serde_yaml::Value` | — |
| **B3 Statements** | **`build_statements` over `&Document` → `Statements` (registry + block→entity + conflicts)** | **B2 (uses `parse_metadata` for canonical metadata)** |
| B4 Argument model + PCS roles | arguments + resolved PCS roles/inference | B3 |
| B5 Relations | resolved, deduped dialectical edges between nodes | B3, B4 |
| B6 Tags / map | tag registry; node+edge map (the `dung` consumer) | B2–B5 |

B3 has a hard dependency on **B2** — the `canonical_metadata` field on a
statement entity is the parsed YAML of the first definition's metadata
block. If a definition has no metadata, `canonical_metadata` is `None`;
if it has metadata, B3 calls `parse_metadata` to produce the `Value`.
B2's `MetadataError` is consumed internally; B3 is total and never
exposes that error to its callers (B2's parse failure on the canonical
metadata becomes `canonical_metadata: None` in the model — B3 is the
model, not a validator).

## Decisions

Three calls drove B3's shape. They are recorded here as the template B4
(argument model) and B6 (graph) will follow, just as B1's flat-arena
decision was the template for B2's flat-module shape.

1. **Identity + canonical content, not minimal-id-only and not
   occurrence-list.** A statement entity is `Statement { id, title,
   canonical_text: Option<String>, canonical_metadata: Option<Value> }`.
   Rationale: B5 (relations) needs the entity's identity, but it also
   wants the canonical text/metadata to render dialectical edges between
   claims — B5 shouldn't have to re-walk the AST to fetch what `[A]`
   says. A bare-id registry (the "A" alternative) would force every
   consumer to re-derive the canonical; an occurrence list (the "C"
   alternative) preserves per-block info B5 doesn't need and pushes a
   `BlockId` concept into B3 that's more naturally B6's job.
2. **First definition wins; later definitions are strict conflicts.** A
   second `[A]: claim` produces a `StatementConflict` recording the
   title, the canonical span, and the conflicting spans — surfaced as
   data on `Statements.conflicts`, not as a `Result` failure. Rationale:
   real Argdown documents occasionally have unintended redefinitions
   (often when refactoring), and silently keeping the first one (the
   "B/lax" alternative) hides the bug from the user. Recording but
   not using (the "C/lax+track" alternative) is a half-step — the
   model has data no consumer reads, which is YAGNI. The
   model/validator split is clean: B3 reports, a future validator
   slice acts. (This breaks the B1/B2 precedent of total-and-quiet:
   B1's `build_sections` didn't surface "heading skipped a level" as
   data either — but statement redefinition is more obviously a
   user-visible semantic error than heading-level skipping.)
3. **`canonical_text: Option<String>`, not always-present `String`.**
   `None` means "the entity is referenced but never defined in this
   document" (e.g., a doc that says `[A]` without any `[A]: ...`).
   `Some("")` would mean "defined as the empty string", which is a
   rare-but-distinct case. Rationale: B5 wants to know "this entity
   has no claim yet" vs "this entity claims nothing" — they're
   different downstream. (The "A/empty-string" alternative loses
   that distinction. The "C/Vec of definitions" alternative is
   over-recording for B3.)

Also settled:

- **An entity exists for every titled statement, even references-only.**
  An entity is created on **first occurrence** (definition or reference
  — whichever comes first in source order). This is the natural read of
  "equivalence class": the class is established by the first time the
  title is mentioned, not by the first definition. The canonical fills
  in later if a definition comes after the first reference.
- **Plain text statements are not entities.** `Statement.title == None`
  blocks (e.g., `just some claim`) get `None` in `block_statements` and
  contribute no entry to the arena. B3 is the equivalence-class model;
  untitled text has no class to be a member of.
- **Inline statement mentions are not entities either.**
  `StatementMention { title }` nodes inside `Statement.inlines` are
  handled by walking the AST at the consumer site (B5 will do this
  when it traverses inlines for relation sources/sinks). B3 handles
  block-level statements only.
- **No new external dependency.** B3 reuses B2's `parse_metadata` and
  `Value` for `canonical_metadata`. The only `std` types it touches are
  `HashMap` (for the title→id map) and `Vec` (for the arena).
- **One function, total, B1-parallel.** `build_statements(&Document) ->
  Statements` is the entire public surface, mirroring B1's
  `build_sections(&Document) -> Sections`.
- **No `Model` aggregate type yet.** Deferred per B1 and B2's
  out-of-scope lists. B4 (argument model) may introduce the first
  aggregate when there's something to aggregate across slices.

## Architecture

A new module `crates/argdown-model/src/statements.rs` in the existing
`argdown-model` crate (B1 and B2's home). Picked up by the existing
`members = ["crates/*"]` workspace glob automatically. **No new
dependency** — B3 reuses B2's `parse_metadata` and the re-exported
`Value` type, plus `std::collections::HashMap` for the title→id map.
`argdown-mcp` is **not** modified in B3.

B3's entire public surface is one pure, total function plus four types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatementId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement { /* ... see Data types */ }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementConflict { /* ... see Data types */ }

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Statements { /* ... see Data types */ }

pub fn build_statements(document: &Document) -> Statements
```

## Data types

```rust
/// Stable, source-order id; indexes `Statements::statements`.
///
/// Stable within a single parse only (the source is re-parsed fresh each
/// time); not designed to survive edits. This matches `SectionId` from B1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatementId(pub usize);

/// One statement entity in the equivalence class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub id: StatementId,
    /// Always `Some` — only titled statements form entities.
    /// Plain-text (untitled) statements are not in the model.
    pub title: String,
    /// First definition's text, or `None` if the entity is referenced but
    /// never defined in this document. `Some("")` (defined as empty) is
    /// distinct from `None` (referenced only).
    pub canonical_text: Option<String>,
    /// First definition's metadata, parsed via B2's `parse_metadata`;
    /// `None` if no definition, or the definition had no metadata block,
    /// or `parse_metadata` returned an error (B3 is total — B2 errors
    /// are absorbed as "no parsed metadata" rather than propagated).
    pub canonical_metadata: Option<Value>,
}

/// A redefinition conflict: a title was defined more than once. Surfaced
/// as data on `Statements::conflicts`, not as a `Result` failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementConflict {
    pub title: String,
    /// Source span of the first (canonical) definition. Note: this span
    /// is recorded even if the entity's `canonical_text` was set later
    /// (e.g., the title's first occurrence was a reference and the first
    /// definition came after) — it points at the definition, not the
    /// first occurrence.
    pub canonical_span: Span,
    /// Source spans of every later (conflicting) definition, in source
    /// order.
    pub conflicting_spans: Vec<Span>,
}

/// The B3 output: a flat statement arena, a block→statement assignment,
/// and a list of redefinition conflicts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Statements {
    /// Flat arena. `StatementId(i)` indexes `statements[i]`. Source order
    /// of first occurrence (definition or reference — whichever comes
    /// first in the document).
    pub statements: Vec<Statement>,
    /// Index-aligned with `document.blocks`: the statement entity for each
    /// titled-statement block, or `None` for plain-text statements and
    /// non-statement blocks (Heading, Argument, Relation, Pcs).
    pub block_statements: Vec<Option<StatementId>>,
    /// Redefinition conflicts found while walking the document, in source
    /// order (sorted by the order their title first appeared).
    pub conflicts: Vec<StatementConflict>,
}
```

The function is **total** (no `Result`) — like `build_sections`. The
strictness shows up as data in `conflicts`, not as a failure mode.

## Algorithm — single pass with a title→id map

```rust
pub fn build_statements(document: &Document) -> Statements {
    use std::collections::HashMap;

    let mut statements: Vec<Statement> = Vec::new();
    let mut by_title: HashMap<String, StatementId> = HashMap::new();
    // Conflicts keyed by title; we'll drain and sort at the end so the
    // output is in source order of the title's first appearance.
    let mut conflict_map: HashMap<String, StatementConflict> = HashMap::new();
    let mut block_statements: Vec<Option<StatementId>> =
        Vec::with_capacity(document.blocks.len());

    for block in &document.blocks {
        // 1. Resolve the block's id (if any).
        let id = match block {
            Block::Statement(s) => s.title.as_ref().map(|title| {
                *by_title.entry(title.clone()).or_insert_with(|| {
                    let id = StatementId(statements.len());
                    statements.push(Statement {
                        id,
                        title: title.clone(),
                        canonical_text: None,
                        canonical_metadata: None,
                    });
                    id
                })
            }),
            _ => None,
        };

        // 2. For a definition, fill in canonical on first occurrence and
        //    record a conflict on a redefinition.
        if let (Block::Statement(s), Some(id)) = (block, id) {
            if !s.is_reference {
                let title = s.title.as_ref().expect("definition has title");
                let entry = &mut statements[id.0];
                if entry.canonical_text.is_none() {
                    entry.canonical_text = Some(s.text.clone());
                    entry.canonical_metadata = s
                        .metadata
                        .as_ref()
                        .map(parse_metadata)
                        .transpose()
                        .ok()
                        .flatten();
                    // The first definition is the canonical — record its
                    // span in the conflict entry (created empty if no
                    // conflict has been recorded for this title yet).
                    let entry = conflict_map.entry(title.clone()).or_insert_with(|| {
                        StatementConflict {
                            title: title.clone(),
                            canonical_span: s.span,
                            conflicting_spans: Vec::new(),
                        }
                    });
                    entry.canonical_span = s.span;
                } else {
                    // Already defined — this is a redefinition conflict.
                    let entry = conflict_map.entry(title.clone()).or_insert_with(|| {
                        StatementConflict {
                            title: title.clone(),
                            canonical_span: s.span,
                            conflicting_spans: Vec::new(),
                        }
                    });
                    entry.conflicting_spans.push(s.span);
                }
            }
        }

        // 3. Record the block→entity mapping.
        block_statements.push(id);
    }

    // Drain conflicts in source order (by first appearance of the title).
    let mut conflicts: Vec<StatementConflict> = conflict_map.into_values().collect();
    conflicts.sort_by_key(|c| {
        statements.iter().position(|s| s.title == c.title).unwrap_or(0)
    });

    Statements {
        statements,
        block_statements,
        conflicts,
    }
}
```

Key invariants:

- **An entity is created on first occurrence** (definition or reference —
  whichever comes first). Source order is determined by first-occurrence
  order, matching B1's section-id pattern.
- **`canonical_text` is `None` until the first definition.** If the first
  occurrence is a reference, the entity has no canonical until a later
  definition fills it in. (This is a deliberate asymmetry: the entity's
  *existence* is established by the first mention, but its *content* is
  established by the first definition.)
- **A second definition does NOT replace the canonical.** It adds a
  `conflicting_span` to the `StatementConflict` for that title. The
  canonical stays the first definition.
- **Plain text statements and non-statement blocks both push `None`.**
  The `block_statements` map is specifically for titled-statement blocks;
  everything else is `None`.
- **B2 errors are absorbed.** If `parse_metadata` returns `Err` on the
  canonical metadata, B3 records `canonical_metadata: None` — the model
  is total, even when the upstream layer is partial. This is consistent
  with B3's "model, not validator" stance: a parse failure on metadata
  is a "we couldn't parse the metadata" data point, not a fatal error.

## Error handling

None. `build_statements` is total. The strictness ("first definition
wins; later definitions are conflicts") is data, not control flow.
`StatementConflict` is a normal field on `Statements`; consumers decide
whether to log it, surface it to a user, or ignore it (e.g., a future
MCP tool that renders docs would show "Warning: `[A]` defined multiple
times").

B2's `parse_metadata` is the only partial function B3 calls, and its
error is absorbed: B3 records `canonical_metadata: None` rather than
propagating. This is the deliberate seam between B2 (partial,
parse-only) and B3 (total, model-only). A future validator slice could
re-parse the canonical metadata and surface B2 errors to the user.

## Testing (TDD)

Failing-test-first per behavior, in the new module, gated by
`cargo test`, `cargo clippy --all-targets -D warnings`, and `cargo fmt`:

1. **Empty document** → `Statements::default()` (empty arena, empty
   `block_statements`, empty `conflicts`).
2. **Single titled definition** (`[A]: claim`) → 1 entity with
   `canonical_text = Some("claim")`, no conflicts, `block_statements[0]
   = Some(id)`.
3. **Single titled reference** (`[A]`) → 1 entity with
   `canonical_text = None`, no conflicts.
4. **Definition + later reference** (`[A]: claim` then `[A]`) → 1
   entity, canonical from the definition, both blocks map to the same
   id.
5. **Reference + later definition** (`[A]` then `[A]: claim`) → 1
   entity, canonical filled in by the later definition, both blocks
   map to the same id.
6. **Redefinition** (`[A]: claim1` then `[A]: claim2`) → 1 entity with
   `canonical_text = Some("claim1")`, 1 conflict with the right spans.
7. **Three different titles** (`[A]: ...`, `[B]: ...`, `[C]: ...`) →
   3 entities in source order, no conflicts.
8. **Plain text statement** (`just some text`) → 0 entities (no title,
   not in the model), `block_statements[0] = None`.
9. **Non-statement blocks** (`# heading`, `<A>: desc`) →
   `block_statements[i] = None` for each, no entities created.
10. **Three redefinitions** (`[A]: c1`, `[A]: c2`, `[A]: c3`) → 1
    entity, 1 conflict with 2 conflicting spans.
11. **Canonical metadata parsed** (definition with `{key: value}`
    metadata) → `canonical_metadata = Some(Value::Mapping{...})` (the
    exact shape is `Value::Mapping` with one entry; the value of the
    `key` entry is `Value::String("value")`).
12. **Title normalization** — the parser's `statement_title` already
    trims whitespace; B3 doesn't re-normalize, so titles with
    surrounding whitespace and titles without are distinct.
13. **Conflicts sorted in source order** — three titles redefined in
    different order than their first appearance; conflicts come out in
    the order the titles first appeared (not in the order the
    redefinition happened).

Tests use `argdown-parser` as dev-dependency to build `Document` inputs
from real Argdown (B1-parallel pattern).

## Out of scope (YAGNI; noted for later slices)

- **A `Model` aggregate type** — introduced when a second slice exists,
  per B1 and B2's out-of-scope lists. B4 (argument model) will
  introduce the first aggregate if needed.
- **Inline statement mentions** (`StatementMention { title }` in
  `Statement.inlines`) — not entities in the model. Consumers find
  them via the AST; B3 only handles block-level statements.
- **A per-statement occurrence list** — what option C in the first
  question would have given. Rejected. B5 will resolve mentions via
  AST-walking when it needs to know "where is `[A]` mentioned?"
- **A `BlockId` concept** — option C in the first question would have
  introduced one. B3 uses `Span` (already in `argdown_core`) for
  location tracking; consumers needing block positions can compute them
  from the AST.
- **Detection of "referenced but never defined"** — that's a validator
  concern, not a model concern. B3 surfaces the data (`canonical_text:
  None`); a future validator slice could iterate `Statements` and
  surface missing definitions.
- **Detection of references to undefined statements** — same as above.
- **Per-section statement lists** (`statements_in_section(SectionId)`)
  — consumers compose `build_sections` and `build_statements` as
  needed. No premature accessor.
- **Typed accessors or query methods on `Statements`** — same as B1
  and B2: the model is data, consumers walk it.

## Summary

B3 is the third slice of Layer B. It does one thing: turn titled
`Block::Statement` instances into an equivalence-class model, with a
strict redefinition rule surfaced as data. One function, no new
external dependency, one new module, B1-parallel structure. The
function is total but produces a `conflicts: Vec<StatementConflict>`
field for consumers to act on. B2's `parse_metadata` is consumed
internally to populate `canonical_metadata`; B2 errors are absorbed
into "no parsed metadata" rather than propagated, keeping B3 a pure
model layer. Out-of-scope items are deferred to B4–B6.
