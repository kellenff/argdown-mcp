# Layer B — PCS Resolution + Model Aggregate (B4b) — Design

- **Date:** 2026-06-06
- **Status:** Draft (for review)
- **Scope:** The second half of Layer B slice B4. B4b resolves the flat
  `Block::Pcs` AST into **roles** (premise / intermediary-conclusion /
  main-conclusion), binds each **inference** to its conclusion, links each PCS
  to its **argument**, and — because reference fidelity demands a complete,
  unified identity space — introduces the first **`Model` aggregate**: the
  complete statement and argument registries (titled + untitled statements;
  named + anonymous arguments) plus the resolved PCSs. Pure and **total**;
  strictness is surfaced as data. `argdown-mcp` remains a placeholder.

## Context

B4 was split into **B4a** (argument equivalence-class model — shipped) and
**B4b** (this slice). B4b's representation is cross-cutting (it governs B5
relations and B6 the Dung map), so it was pressure-tested with a **multi-model
chorus brain-jam** (OpenAI + Gemini + MiniMax, 3 rounds), then the contested
points were settled by **probing the reference implementation `@argdown/core`
via the argdown MCP** (`export_json`). The brain-jam fixed the internal
representation; the reference probing corrected the linkage model and drove the
full-fidelity decisions below.

### Decisions settled this session (auto-logged)

1. **Unified item sequence** for a resolved PCS (brain-jam load-bearing call):
   one `Vec<ResolvedPcsItem>` preserving source-order topology — *not* parallel
   role vectors, *not* split statement/inference vectors.
2. **Reference-faithful fidelity**: B4b mints anonymous arguments (for
   standalone PCSs) and untitled-statement equivalence classes, matching
   `@argdown/core`.
3. **B4b introduces the first `Model` aggregate** (as B3's spec foresaw) that
   completes B3's statement registry and B4a's argument registry. B3 and B4a
   stay untouched; the Model is additive and composes them.

## Reference semantics (empirical — from `@argdown/core` probes)

Seven `export_json` probes established the ground truth B4b must reproduce:

- **Roles are positional.** A PCS statement not preceded by an inference is a
  `premise`. A statement immediately following an inference is a conclusion;
  the **last** conclusion is `main-conclusion`, earlier ones are
  `intermediary-conclusion`. (Confirmed with a two-step PCS.)
- **Inference binds forward.** Each inference carries `inferenceRules:
  Vec<String>` and binds to the conclusion that follows it.
- **Linkage is strict adjacency.** A PCS attaches to a named argument **iff
  that argument block is the immediately-preceding block** (definition `<A>:
  …` *or* bare reference `<A>`). *Any* intervening block — a statement
  definition, a section heading, a relation, or another PCS — **detaches** it.
  (Probes: `<A>`·section·PCS and `<A>`·statement·PCS both left `A.pcs == []`
  and produced a standalone PCS.)
- **No orphans; standalone PCS = anonymous argument.** A detached/standalone
  PCS is not an error — `@argdown/core` makes it a new **anonymous argument**
  (auto-titled "Untitled N"). A second PCS after one argument becomes its own
  anonymous argument. **Every PCS belongs to exactly one argument; an argument
  has at most one PCS.**
- **Untitled statements are singletons.** Each untitled PCS statement becomes
  its own equivalence class ("Untitled N"); they never merge. Titled PCS
  statements merge by title with each other and with top-level statements of
  the same title.
- **Interspersed relations are transparent.** A relation inside a PCS (a child
  of a statement) does not break inference→conclusion binding; it attaches to
  the statement's equivalence class.

These corrected the brain-jam's linkage assumptions (it had proposed a
skip-tolerant scan, an `OrphanedPcs` issue, and "first-PCS-wins" — all wrong)
and **vindicated the skeptic's `ResolvedStatementId`/identity idea** (needed
once untitled statements are first-class).

## Decisions

1. **Unified `Vec<ResolvedPcsItem>`.** `Statement { role, number, statement,
   span }` | `Inference { rules, concludes_item_idx, span }` |
   `Relation(Relation)` (pass-through for B5). The enum variant *is* the
   role/binding. `concludes_item_idx: Option<usize>` is an absolute index into
   the same `items` vector (`None` when degenerate). Relations are transparent
   to binding. Contract: `items` indices are stable; consumers must not filter
   in place.
2. **Every PCS has an owning argument** — `ResolvedPcs.argument: ArgumentId`
   (not `Option`). It is a named B4a argument when one is immediately
   preceding, else a freshly-minted **anonymous argument**.
3. **Every PCS statement resolves to a Model statement** —
   `ResolvedPcsItem::Statement.statement: StatementId` (not `Option`). Titled
   statements merge by title (with each other and top-level); untitled ones are
   minted as singletons.
4. **Strictness as data.** `PcsIssue` for genuine internal malformations only:
   `InferenceWithNoConclusion`, `ConsecutiveInferences`, `EmptyPcs`. Statement
   and argument redefinition conflicts (now spanning top-level + PCS
   definitions) surface as `StatementConflict` / `ArgumentConflict` (B3/B4a
   types reused). No `Result`.
5. **Additive Model.** The Model reuses B3's `StatementId` and B4a's
   `ArgumentId` newtypes; its arenas are **supersets whose prefixes equal
   B3's / B4a's** (same id → same entity), with new ids appended for PCS-only
   titled statements, untitled singletons, and anonymous arguments. B3/B4a
   source is unchanged.
6. **Signature** `build_model(&Document, &Arguments, &Statements) -> Model` —
   reuses B3/B4a outputs as the seed (no re-derivation), resolves all
   cross-links in one pass.

## Architecture

A new module `crates/argdown-model/src/model.rs`. **No new external
dependency** (reuses `argdown_core`, B2's `parse_metadata`/`Value`, B3/B4a, and
`std::collections::HashMap`). `lib.rs` gains `mod model;` and a `pub use`. B3
(`statements.rs`), B4a (`arguments.rs`), B2, B1 are untouched. `argdown-mcp` is
not modified.

The Model owns its own entity types (statements need an optional title, which
B3's always-titled `Statement` cannot express), but reuses the `StatementId` /
`ArgumentId` newtypes so B5/B6 use one id space.

## Data types

```rust
use argdown_core::{Block, Document, Relation, Span};

pub use crate::arguments::ArgumentId;
pub use crate::metadata::Value;
pub use crate::statements::{ArgumentConflict, StatementConflict, StatementId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcsId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Premise,
    IntermediaryConclusion,
    MainConclusion,
}

/// One resolved item in a PCS, in source order. The enum variant carries the
/// item's resolved role/binding. `items` indices are stable for the lifetime
/// of the `ResolvedPcs`; consumers MUST NOT filter or reorder in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedPcsItem {
    Statement {
        role: Role,
        number: usize,
        /// The Model statement this occurrence resolves to (titled → merged
        /// by title; untitled → its minted singleton). Always resolvable.
        statement: StatementId,
        span: Span,
    },
    Inference {
        rules: Vec<String>,
        /// Absolute index into the parent `items` vector of the conclusion
        /// statement. `None` when degenerate (trailing inference, consecutive
        /// inferences). Relations between an inference and its conclusion are
        /// transparent — the next Statement in source order is the conclusion.
        concludes_item_idx: Option<usize>,
        span: Span,
    },
    /// Pass-through: B5 resolves relation targets; B4b preserves position so
    /// B5 has structural context without re-walking the AST.
    Relation(Relation),
}

/// A resolved premise-conclusion structure, owned by exactly one argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPcs {
    pub id: PcsId,
    /// Owning argument — a named B4a argument (immediately preceding) or a
    /// minted anonymous argument. Never absent.
    pub argument: ArgumentId,
    pub items: Vec<ResolvedPcsItem>,
    pub span: Span,
}

/// Complete statement equivalence class. `title: None` ⇔ an untitled PCS
/// statement (a singleton). Prefix-correspondence: `Model::statements[i]` for
/// `i < statements.len()` (B3's count) equals B3's `StatementId(i)` entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelStatement {
    pub id: StatementId,
    pub title: Option<String>,
    pub canonical_text: Option<String>,
    pub canonical_metadata: Option<Value>,
}

/// Complete argument entity. `title: None` ⇔ an anonymous argument (a
/// standalone PCS). Prefix-correspondence with B4a's `Arguments` arena.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelArgument {
    pub id: ArgumentId,
    pub title: Option<String>,
    pub canonical_description: Option<String>,
    pub canonical_metadata: Option<Value>,
    /// The argument's PCS, if any. For anonymous arguments, always `Some`.
    pub pcs: Option<PcsId>,
}

/// Genuine internal PCS malformations, surfaced as data (never `Result`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PcsIssue {
    InferenceWithNoConclusion { inference_span: Span },
    ConsecutiveInferences { first_span: Span, second_span: Span },
    EmptyPcs { pcs_span: Span },
}

/// The first Layer-B `Model` aggregate: complete registries + resolved PCSs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Model {
    /// Complete statement registry (titled top-level + titled-in-PCS merged by
    /// title + untitled singletons). Prefix equals B3's arena.
    pub statements: Vec<ModelStatement>,
    /// Complete argument registry (named + anonymous). Prefix equals B4a's.
    pub arguments: Vec<ModelArgument>,
    /// Resolved PCSs, in source order.
    pub pcs: Vec<ResolvedPcs>,
    /// Index-aligned with `document.blocks`: the PCS for each `Block::Pcs`,
    /// else `None`.
    pub block_pcs: Vec<Option<PcsId>>,
    /// Redefinition conflicts over the complete (top-level + PCS) definition
    /// set. Supersets B3's / B4a's.
    pub statement_conflicts: Vec<StatementConflict>,
    pub argument_conflicts: Vec<ArgumentConflict>,
    /// Internal PCS malformations.
    pub issues: Vec<PcsIssue>,
}

pub fn build_model(
    document: &Document,
    arguments: &Arguments,
    statements: &Statements,
) -> Model
```

## Algorithm

A multi-pass walk over `document.blocks`, seeded from B3/B4a:

1. **Seed registries.** Copy B3's `statements` into `Model.statements`
   (`title: Some`) and B4a's `arguments` into `Model.arguments`, preserving ids
   (prefix-correspondence). Carry forward B3/B4a conflicts. Maintain
   `title → StatementId` and `title → ArgumentId` maps from the seeds.

2. **Linkage pass.** For each `Block::Pcs` at index `i`, inspect block `i-1`
   (the immediately-preceding block; blanks/comments are not blocks in our
   AST): if it is a `Block::Argument`, the PCS is owned by that argument's
   `ArgumentId` (found via the title map); otherwise **mint an anonymous
   argument** (`title: None`) and own the PCS with it. Set the owning
   argument's `pcs = Some(pcs_id)`. Record `block_pcs[i] = Some(pcs_id)`.

3. **Statement resolution.** For each `PcsItem::Statement`, resolve its
   `StatementId`: if titled, find-or-create the class by title (merging with
   top-level and other PCS occurrences; a definition fills canonical or records
   a `StatementConflict`, mirroring B3); if untitled, mint a singleton
   `ModelStatement { title: None, canonical_text: Some(text), … }`.

4. **Role + inference resolution** (per PCS, single linear pass mirroring the
   brain-jam algorithm): track `pending_inference_idx`. On a `Statement`
   following a pending inference, bind it (`concludes_item_idx`) and clear
   pending; otherwise it is a `Premise`. On an `Inference` while one is pending,
   emit `ConsecutiveInferences` and leave the prior `concludes_item_idx = None`.
   `Relation` items are transparent. After the loop, a still-pending inference
   yields `InferenceWithNoConclusion`. An empty `items` yields `EmptyPcs`.

5. **Role finalization.** Among a PCS's bound conclusions, the last in `items`
   order is `MainConclusion`; earlier ones `IntermediaryConclusion`; all unbound
   statements are `Premise`.

The function is **total**; every branch produces data, never a `Result`.

## Error handling

None. `build_model` is total. `parse_metadata` errors are absorbed
(`canonical_metadata: None`), as in B3/B4a. Malformations are data
(`issues`, `*_conflicts`).

## Testing (TDD)

Failing-test-first per behavior; gated by `cargo test`, `cargo clippy -D
warnings`, `cargo fmt`. Tests use `argdown-parser` to build inputs and assert
the resolved model against the **reference probes** captured above. Coverage:

- **Roles:** single-step PCS (premise/main); two-step PCS (premise →
  intermediary → premise → main); premises-only (no inference).
- **Inference:** rules captured; `concludes_item_idx` correct; bare `----`
  (empty rules) vs `-- Rule, Rule --`.
- **Linkage (reference-faithful):** `<A>:`·PCS attaches; bare `<A>`·PCS
  attaches; `<A>`·section·PCS detaches (anonymous arg); `<A>`·statement·PCS
  detaches; two PCSs after one argument → second is anonymous; PCS with no
  preceding argument → anonymous.
- **Completion:** titled PCS statement merges with a top-level statement of the
  same title (one `StatementId`); titled PCS statement appearing only in PCSs
  gets its own class; untitled PCS statements are distinct singletons; anonymous
  arguments appear in `Model.arguments` with `title: None` and `pcs: Some`.
- **Conflicts:** a title defined top-level and re-defined in a PCS records a
  `StatementConflict`.
- **Degenerate (issues):** trailing inference → `InferenceWithNoConclusion`;
  consecutive inferences → `ConsecutiveInferences`; interspersed relation does
  **not** break binding (transparent).
- **Index alignment:** `block_pcs` length equals `document.blocks` length;
  non-PCS blocks map to `None`.
- **Prefix-correspondence:** `Model.statements`/`arguments` prefixes equal the
  B3/B4a inputs (same ids, same entities).

## Out of scope (deferred)

- **Relation resolution** (`PcsItem::Relation` is pass-through) — B5.
- **The Dung node+edge map / tags** — B6.
- **Exact `@argdown/core` "Untitled N" title strings** — identities are by id
  (titled-by-title, untitled/anonymous by minted id); display labels are
  derivable and not stored.
- **Inline statement/argument mentions** in bodies — consumers walk the AST.

## Summary

B4b resolves PCS roles and inference binding via a unified source-order item
sequence, links each PCS to its argument by strict adjacency (minting anonymous
arguments for standalone PCSs), and introduces the first `Model` aggregate —
the complete, unified statement and argument registries the reference produces
and that B5/B6 need. One new module, no new dependency, additive over B3/B4a.
Total function; malformations and conflicts are data. The design is grounded in
a multi-model chorus brain-jam and seven `@argdown/core` reference probes.
