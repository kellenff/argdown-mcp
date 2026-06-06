# Layer B — Relations (B5) — Design

- **Date:** 2026-06-06
- **Status:** Draft (for review)
- **Scope:** B5 resolves the flat `Relation` AST into **deduped, directed
  dialectical edges between nodes**, extending the B4b `Model` aggregate. It
  adds `Node`, `RelationKind`, `Edge`, and an `edges` field to `Model`, mints
  any relation-participant nodes the registry is still missing, and resolves
  each relation's source by a monotonic indent stack. Pure and **total**;
  malformations are data. `argdown-mcp` remains a placeholder.

## Context

B5 is "resolved, deduped dialectical edges between nodes" (depends on B3, B4).
Decided this session: **B5 extends the `Model` in a single slice** (not a
standalone function), and the edge/node representation was settled by a
**multi-model chorus brain-jam** on top of reference (`@argdown/core`) probes.

### Reference semantics (probed)

- **Source = nearest enclosing element by indent.** A relation nested under
  another relation has the **enclosing relation's target** as its source
  (probe: `- plain attack on B` under `+ [B]` ⇒ *Untitled1 attacks B*).
- **Direction:** Inbound (`+`,`<+`,`-`,`<-`,`_`,`<_`) ⇒ edge **target→parent**;
  Outbound (`+>`,`->`,`_>`) ⇒ **parent→target**; Contradictory (`><`) ⇒
  symmetric, recorded once as parent→target.
- **Targets:** titled statements merge by title; plain/untitled targets become
  singleton equivalence classes (like untitled PCS statements); argument
  targets are argument nodes.
- **Dedup by `(from, to, kind)`:** `[A]\n  + [B]` and `[B]\n  +> [A]` both mean
  *B supports A* ⇒ **one** edge.
- **Relations must be indented** to attach — an unindented `+ [B]` after `[A]`
  is a parse error in the reference. (Our parser carries `indent` on `Relation`
  only; `Statement`/`Argument`/`PcsItem::Statement` have no indent and sit at
  the base level.)

## Decisions (chorus-settled)

1. **`Node` = `Statement(StatementId) | Argument(ArgumentId)`.** Undercut is kept
   as an edge `kind`; representing an inference/edge *as a node* (for
   undercut-the-inference) is deferred to B6's Dung compiler — B5 records the
   `Undercut` edge faithfully and loses nothing.
2. **`Edge { from: Node, to: Node, kind: RelationKind, span: Span }`**, with
   `RelationKind = Support | Attack | Undercut | Contradictory`. Contradictory
   is a single directed edge; consumers treat it as symmetric (no flag, no
   doubled edge).
3. **`Model.edges: Vec<Edge>`**, deduped by `(from, to, kind)` in
   first-occurrence source order (the first occurrence's `span` is kept).
4. **Monotonic indent stack, peek-then-enter.** A relation reads its parent
   from the stack *before* pushing; then pushes its own target so deeper
   relations nest under it. (The brain-jam's first cut pushed a placeholder and
   self-parented; the skeptic caught it.)
5. **Node completeness.** B5 mints nodes for every relation participant the
   registry lacks: titled statements/arguments merge by title; plain/untitled
   statements become fresh singletons. The `Model` becomes the complete node
   registry; prefix-correspondence with B3/B4a/B4b is preserved (existing ids
   unchanged; new nodes appended).
6. **`RelationIssue::RelationWithoutParent { span }`** on a new
   `Model.relation_issues` field — a relation with no enclosing parent. Total;
   no `Result`, no fabricated nodes.

## Data types (added to `model.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node {
    Statement(StatementId),
    Argument(ArgumentId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    Support,
    Attack,
    Undercut,
    Contradictory,
}

/// A resolved directed dialectical edge. `Contradictory` is symmetric but
/// stored once (from = the parent, to = the target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: Node,
    pub to: Node,
    pub kind: RelationKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationIssue {
    /// A relation with no enclosing element to attach to.
    RelationWithoutParent { span: Span },
}

// Model gains:
//   pub edges: Vec<Edge>,
//   pub relation_issues: Vec<RelationIssue>,
```

`RelationOperator` → `RelationKind` and `RelationDirection` → orientation are
mapped 1:1 (the `+`≡`<+` collapse already happened in the parser).

## Algorithm (folded into `build_model`)

A `Frame { indent: usize, node: Node }` stack, maintained by an `enter` helper:

```rust
fn enter(stack, indent, node) {
    while stack.last().is_some_and(|f| f.indent >= indent) { stack.pop(); }
    stack.push(Frame { indent, node });
}
```

Walking `document.blocks` (extending the existing B4b pass):

- **`Block::Statement`** → resolve its node (titled: by-title; plain: mint a
  singleton) and `enter(stack, 0, node)` — top-level statements/arguments sit at
  base indent 0 (they carry no indent).
- **`Block::Argument`** → `enter(stack, 0, Node::Argument(id))`.
- **`Block::Pcs`** → after B4b resolution, resolve the PCS's interspersed
  relations in a **PCS-local scope**: each `PcsItem::Statement` is the base node
  for following relations; each `PcsItem::Relation` resolves via the same
  peek-then-enter rule against a PCS-local stack seeded with the PCS statements.
  (PCS relations and top-level relations never share a stack — sidestepping the
  coordinate-space concern the skeptic raised, since PCS statements have no
  document-absolute indent.)
- **`Block::Relation(r)`**:
  ```
  parent = stack.iter().rev().find(|f| f.indent < r.indent)   // peek
  if none → relation_issues.push(RelationWithoutParent{r.span}); continue
  target = resolve_target(&r.target)        // merge/mint node
  (from, to) = orient(parent, target, r.direction)
  push_if_new(&mut edges, Edge{from, to, kind: r.operator.into(), span: r.span})
  enter(stack, r.indent, target)            // deeper relations nest under target
  ```

`orient`: Inbound ⇒ `(target, parent)`; Outbound/Bidirectional ⇒
`(parent, target)`. `push_if_new` dedups on `(from, to, kind)`.

`resolve_target`: `RelationTarget::Statement` → titled by-title merge / plain
singleton; `RelationTarget::Argument` → by-title merge or mint (arguments are
always titled).

## Error handling

None — total. A parentless relation → `RelationWithoutParent` data; everything
else resolves. B2 metadata errors absorbed as before.

## Testing (TDD)

Against the reference probes; gated by `cargo test`/`clippy -D warnings`/`fmt`:

- **Direction:** `[A]\n  + [B]` ⇒ edge B→A Support; `[A]\n  +> [B]` ⇒ A→B;
  `[A]\n  - [B]` ⇒ B→A Attack; `[A]\n  >< [B]` ⇒ one Contradictory edge.
- **Nesting / source:** `[A]\n  + [B]\n    - [C]` ⇒ B→A Support **and** C→B
  Attack (nested source = enclosing target).
- **Targets:** plain target ⇒ a minted singleton node is the `from`; `<Arg>`
  target ⇒ `Node::Argument`; titled target merges with a top-level same-title.
- **Dedup:** `[A]\n  + [B]` plus `[B]\n  +> [A]` ⇒ exactly one B→A Support edge.
- **PCS-interspersed:** `(1) [P]\n  +> [S]\n----\n(2) C` ⇒ an edge P→S, and the
  PCS roles still resolve (B4b unaffected).
- **Parentless:** an orphan top-level/indented relation ⇒ `RelationWithoutParent`,
  no edge, no panic.
- **Node completeness / prefix-correspondence:** relation-target statements
  appear in `Model.statements`; B3/B4a/B4b prefixes unchanged.

## Out of scope (B6)

Undercut→inference auxiliary-node compilation, per-node adjacency index, edge
ids, the Dung node+edge map, and tags.

## Summary

B5 extends `build_model` to resolve relations into deduped directed `Edge`s
between `Node`s, minting any missing participant nodes so the `Model` is the
complete node+edge registry. Reference-faithful (source-by-indent, direction
normalization, dedup), total (parentless relations are data), additive. Settled
by a chorus brain-jam over reference probes.
