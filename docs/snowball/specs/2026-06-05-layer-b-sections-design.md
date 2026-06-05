# Layer B — Sections (B1) — Design

- **Date:** 2026-06-05
- **Status:** Approved
- **Scope:** The first slice of Layer B (the semantic-assembly layer the parser
  has deferred to since A1). B1 turns the flat parsed `Document` into a nested
  **section tree** plus a **block→section assignment**, in a new `argdown-model`
  crate, computed purely from `&Document` (the AST is never mutated).
  Representation-only: unit-tested in isolation; `argdown-mcp` stays a
  placeholder; the MCP protocol layer remains separately deferred.

## Context

The parser (A1–A5b) emits a deliberately flat AST —
`Document { blocks: Vec<Block>, frontmatter: Option<Metadata> }` — and defers
all structural/semantic assembly to "Layer B." Layer B is not one feature: it
spans sections, statement equivalence classes, the argument/PCS model, relation
resolution, metadata/YAML parsing, and a tags/map layer. Per the project's
"ship thin vertical slices, split when scope balloons" principle, Layer B is
decomposed into slices, each its own spec→plan→build:

| Slice | Produces | Depends on |
| ----- | -------- | ---------- |
| **B1 Sections** | nested section tree + block→section assignment | — |
| B2 Metadata/YAML | `parse_metadata` for element `{…}` + frontmatter | — |
| B3 Statement model | statement equivalence classes | — |
| B4 Argument model + PCS roles | arguments + resolved PCS roles/inference | B3 |
| B5 Relations | resolved, deduped dialectical edges between nodes | B3, B4 |
| B6 Tags / map | tag registry; node+edge map (the `dung` consumer) | B2–B5 |

This spec covers **B1 only**, chosen as the foundational first slice (smallest,
zero-dependency, mirrors how A1 was the parser "spine"). B1's representation
choice deliberately sets the pattern the later slices follow.

The reference `@argdown/core` model (probed via the argdown MCP) keeps a section
**tree** plus a block→section reference **by id** — it does not physically nest
blocks inside section nodes. Probed behavior B1 matches:

- A heading nests under the nearest preceding heading with a **strictly smaller
  level number** (so `#` then `#####` nests the level-5 directly under the
  level-1; skipped levels don't matter).
- Each block is assigned the **most-recently-opened** section; blocks **before
  the first heading belong to no section**.

## Decisions

The representation was the cross-cutting call (it governs B2–B6). It was
pressure-tested with a second model (M2 brain-jam — the A3/A4/A5a precedent) and
settled as **a new `argdown-model` crate emitting an owned, flat section model
computed from `&Document`**, refined by the jam in two ways:

1. **Owned output, not a borrowed `&Document` view.** `build_sections` returns
   an owned `Sections` (no lifetime), so later slices can accumulate onto the
   model without fighting borrows. (Rejected: a borrowed `SectionTree<'a>`.)

2. **Stable, source-derived ids carried forward by value.** `SectionId` is a
   source-order index, stable *within a parse*. Later slices (e.g. B5 relations)
   reference sections by this value, not by re-deriving membership — the
   "single canonical traversal, ids carried as literal values" pattern. Because
   the source is re-parsed fresh every time, cross-edit id stability
   (content-addressing) is **out of scope** (YAGNI).

Also settled:

- **Flat arena over nested structs.** `Vec<Section>` with `parent`/`children` as
  ids, not a tree of owned `Section` structs — honors the flat-over-nested
  throughline and keeps blocks flat. (Rejected: Approach 3, a
  `SectionedDocument` that physically nests blocks inside section nodes — it
  fights the accumulating-layers pattern and loses the flat source-order view
  later slices need.)
- **A dedicated crate over a `model` module in `argdown-core`.** The model grows
  a lot through B2–B6; keeping it out of `core` preserves core's role as "the
  syntax-tree types the parser produces." (Rejected: Approach 2.)
- **The AST is never mutated.** No `parent`/`section` pointers are added to
  `Block`; Layer B owns its derived structure ("spans/views computed from
  source, not baked in").

## Architecture

A new crate `crates/argdown-model/` depending on `argdown-core`. It is picked up
automatically by the existing `members = ["crates/*"]` workspace glob, and added
to `[workspace.dependencies]` so future crates reference it via
`{ workspace = true }`, matching the existing convention. `argdown-mcp` is **not**
modified in B1.

B1's entire public surface is one pure, total function:

```rust
pub fn build_sections(document: &Document) -> Sections
```

## Data types

```rust
/// Stable, source-order id; indexes `Sections::sections`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionId(pub usize);

/// One heading-delimited section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub id: SectionId,
    pub level: u8,                 // 1..=6, from the heading
    pub title: String,             // heading text
    pub heading_span: Span,        // the heading's source span
    pub parent: Option<SectionId>,
    pub children: Vec<SectionId>,  // child sections, in source order
}

/// The B1 output: a flat section arena + the section forest + a
/// block→section assignment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sections {
    /// Flat arena. `SectionId(i)` indexes `sections[i]`. Source order.
    pub sections: Vec<Section>,
    /// Top-level sections (the forest entry points), in source order.
    pub roots: Vec<SectionId>,
    /// Index-aligned with `document.blocks`: the section directly containing
    /// each block, or `None` for blocks before the first heading.
    pub block_sections: Vec<Option<SectionId>>,
}
```

`sections` is the flat arena (the jam's arena-lite shape); navigation is by id.
`block_sections[i]` corresponds to `document.blocks[i]` by position — alignment
is guaranteed by construction (built in the same pass). A heading block is
assigned to the section it opens.

## Build algorithm — one canonical traversal

A single pass over `document.blocks`, maintaining a **stack of open section ids**
and a **current section** (`Option<SectionId>`, initially `None`):

- **`Block::Heading(h)`** at block index `i`:
  1. Pop the stack while the top section's `level >= h.level`.
  2. `parent` = the new top of stack (if any).
  3. Allocate `SectionId(sections.len())`; push a `Section` with
     `level = h.level`, `title = h.text.clone()`, `heading_span = h.span`,
     `parent`, empty `children`.
  4. Record the new id in the parent's `children` (or in `roots` if `parent` is
     `None`).
  5. Push the new id onto the stack; set `current = Some(new id)`;
     `block_sections[i] = current`.
- **Any other block** at index `i`: `block_sections[i] = current`.

This reproduces the probed reference behavior:

- **Level skip** (`#` then `#####`): step 1 pops nothing past the `#`, so the
  `#####` nests directly under the `#`.
- **Pre-first-heading blocks:** `current` is `None` → assigned `None`.
- **No headings:** `sections` and `roots` empty; every `block_sections` entry
  `None`.
- **Frontmatter:** not a block → does not participate.

## Error handling

None. `build_sections` is total. The parser already validated the document
(headings are guaranteed level 1–6), so there is no failure mode; the function
returns `Sections` directly, not `Result`.

## Testing (TDD)

Failing-test-first per behavior, in the new crate, gated by `cargo test`,
`cargo clippy --all-targets -D warnings`, and `cargo fmt`:

1. empty document → `Sections::default()` (empty arena, empty roots, empty
   block_sections);
2. single heading + one following block → one section; both the heading block
   and the following block assigned to its `SectionId`;
3. nested `#` / `##` / `###` → correct `parent`, `children`, and a single root;
4. level skip `#` / `#####` → the `#####` section's `parent` is the `#` section;
5. two `#` siblings (with content between) → two entries in `roots`, correct
   sibling structure;
6. content before the first heading → those blocks map to `None`;
7. a document with no headings → all `block_sections` are `None`;
8. multiple blocks under one heading → all share that heading's `SectionId`.

Nesting and assignment are cross-checked against the exact shapes probed from
`@argdown/core` so B1 tracks the reference.

## Out of scope (YAGNI; noted for later slices)

- Content-addressed / edit-stable ids (we re-parse fresh; ids are
  parse-local).
- A unified cross-layer id scheme — B5 (relations) settles how `SectionId`,
  statement ids, and argument ids coexist.
- Per-section block *lists*: `block_sections` is the source of truth; a
  `blocks_in(SectionId)` accessor can be added when a consumer needs it.
- A top-level `Model` aggregate type — introduced when a second slice exists.
- Any MCP exposure (the MCP protocol layer is a separate deferred effort).
