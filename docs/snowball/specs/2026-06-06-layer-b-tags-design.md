# Layer B — Tags (B6a) — Design

- **Date:** 2026-06-06
- **Status:** Approved
- **Scope:** The first half of Layer B slice B6. B6a builds the document's
  **tag registry** — the unique `#tag`s used anywhere in the document, in
  first-occurrence order — plus a per-block tag assignment, in a new
  `argdown_model::tags` module via `build_tags(&Document) -> Tags`. Pure and
  **total**. Representation-only; `argdown-mcp` stays a placeholder. The Dung
  AF map + grounded extension is the separate B6b half.

## Context

B6 ("Tags / map") is the final Layer B slice (depends on B2–B5). It was
**split** on the established precedent (A2a/A2b, A5a/A5b, B4a/B4b): **B6a** is
the tag registry (low risk, mechanical); **B6b** is the Dung AF map + grounded
extension (cross-cutting — settled via a chorus brain-jam).

Tags are **inline elements only** — `InlineKind::Tag { tag: String }` inside a
`Statement.inlines` or `Argument.inlines` (`Vec<Inline>`). There is no top-level
tag node, and tag text stays in the body (overlay, not stripped — the A4
philosophy). Tags can therefore appear at every site a statement or argument
body appears: top-level `Block::Statement` / `Block::Argument`, the numbered
statements inside a `Block::Pcs`, and the targets of relations (top-level
`Block::Relation` and `PcsItem::Relation`).

Reference (`@argdown/core`, probed via `export_json`): a top-level `tags`
registry keyed by tag name with a first-occurrence `occurrenceIndex` (plus a
display-only `cssClass`), and each statement/argument carries its own
`tags: [...]` list — all sourced from inline `#tag`s. A `{tags: [...]}`
**metadata key was NOT promoted** by default (only inline `#foo` reached the
entity's tags) — so "tags-promotion" is config-gated in the reference and is
**deferred** (see Out of scope).

## Decisions

1. **Registry + per-block assignment, both from inline tags.** `Tags { tags:
   Vec<String>, block_tags: Vec<Vec<TagId>> }`. `tags` is the unique tag names
   in **first-occurrence (source) order** (mirrors the reference's
   `occurrenceIndex`; the display-only `cssClass` is dropped). `block_tags` is
   index-aligned with `document.blocks`: every tag appearing **anywhere within**
   that top-level block (including its PCS items and relation targets),
   deduped, in source order. This matches the project's index-aligned-`Vec`
   template (`block_sections` / `block_statements` / `block_pcs`) and is
   complete — every tag is within exactly one top-level block.
2. **Total, `&Document`-only, no new dependency.** `build_tags(&Document) ->
   Tags`, like every Layer-B slice. Reuses only `std::collections::HashMap`.
3. **Per-equivalence-class aggregation is deferred.** The reference also hangs
   tags off each equivalence class (`class.tags`). B6a's `block_tags` is
   per-block, not per-entity; a consumer composes it with B3/B4a's
   block→entity maps (or B6b's Model) to aggregate onto `StatementId` /
   `ArgumentId`. Keeping B6a per-block avoids re-deriving the registries and
   keeps the slice thin.

## Architecture

A new module `crates/argdown-model/src/tags.rs`. **No new dependency.** `lib.rs`
gains `mod tags;` + a `pub use`. `argdown-mcp` untouched.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub usize);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tags {
    /// Unique tag names in first-occurrence (source) order; `TagId(i)` indexes this.
    pub tags: Vec<String>,
    /// Index-aligned with `document.blocks`: the tags appearing anywhere within
    /// each top-level block (its own body, its PCS items, its relation targets),
    /// deduped, in source order.
    pub block_tags: Vec<Vec<TagId>>,
}

pub fn build_tags(document: &Document) -> Tags
```

## Algorithm

Single pass over `document.blocks`. For each block, collect the `InlineKind::Tag`
strings from every statement/argument body it contains, registering each in the
global `tags` Vec (via a `HashMap<String, TagId>` for first-occurrence ids) and
appending its `TagId` to that block's deduped list:

- `Block::Statement(s)` → `s.inlines`
- `Block::Argument(a)` → `a.inlines`
- `Block::Relation(r)` → the inlines of `r.target` (Statement or Argument)
- `Block::Pcs(p)` → for each item: `PcsItem::Statement { statement }` →
  `statement.inlines`; `PcsItem::Relation(r)` → `r.target` inlines;
  `PcsItem::Inference` → none
- `Block::Heading` → none (headings carry no inlines)

Document frontmatter is skipped (its tags would be metadata-promotion).

## Error handling

None — total. No `Result`.

## Testing (TDD)

Failing-test-first; gated by `cargo test` / `clippy -D warnings` / `fmt`:

1. Empty document → `Tags::default()`.
2. Inline tags on a statement (`[A]: a #foo #bar`) → `tags == ["foo","bar"]`,
   `block_tags == [[0,1]]`.
3. Tags on an argument (`<A>: arg #foo`) → `["foo"]`, `[[0]]`.
4. First-occurrence ordering across blocks (`[A]: a #foo` then `[B]: b #bar
   #foo`) → `["foo","bar"]`, `block_tags == [[0],[1,0]]`.
5. Dedup within a block (`[A]: a #foo #foo`) → `["foo"]`, `[[0]]`.
6. Tag inside a PCS numbered statement → registered, on that PCS block.
7. Tag on a relation target (`[A]: a` then `  + [B]: b #foo`) → `["foo"]`,
   `block_tags == [[], [0]]`.
8. No tags → empty `tags`, all-empty `block_tags`.
9. `block_tags.len() == document.blocks.len()` (index alignment).

Tests use `argdown_parser::parse` (B1–B5-parallel pattern).

## Out of scope

- **Metadata `tags:` promotion** — config-gated in the reference (not default);
  deferred. If added later, it reads the `tags` key from B2's parsed
  `Value` and merges into the registry.
- **Per-equivalence-class tag aggregation** (`class.tags`) — a consumer
  composes `block_tags` with B3/B4a maps.
- **`cssClass` / display concerns** — not modelled.
- **The Dung AF map + grounded extension** — B6b.

## Summary

B6a is the tag half of B6: a complete, total `build_tags(&Document) -> Tags`
collecting inline `#tag`s into a first-occurrence-ordered registry plus a
per-block assignment, walking every statement/argument body site. One new
module, no new dependency, B-slice-parallel structure. Metadata promotion and
per-entity aggregation are deferred; the Dung map is B6b.
