//! PCS resolution + the Model aggregate (Layer B, slice B4b).
//!
//! Resolves each flat `Block::Pcs` into roles (premise / intermediary- /
//! main-conclusion), binds each inference to its conclusion, and links each
//! PCS to its owning argument by **strict adjacency** (a PCS attaches to a
//! named argument only when that argument block is the immediately-preceding
//! block; otherwise the PCS is a standalone *anonymous* argument). Because the
//! reference (`@argdown/core`) treats untitled PCS statements and standalone
//! PCSs as first-class entities, this slice introduces the first Layer-B
//! [`Model`] aggregate: the complete, unified statement and argument registries
//! (titled + untitled; named + anonymous) plus the resolved PCSs.
//!
//! Pure and total — strictness is data ([`Model::issues`],
//! [`Model::statement_conflicts`], [`Model::argument_conflicts`]), never a
//! `Result`. Additive over B3/B4a, whose sources are untouched: `build_model`
//! reuses [`crate::build_statements`] and [`crate::build_arguments`] for the
//! seed registries and preserves their ids (prefix-correspondence).

use argdown_core::{
    Block, Document, PcsItem, Relation, RelationDirection, RelationOperator, RelationTarget, Span,
    Statement,
};
use std::collections::HashMap;

pub use crate::arguments::{ArgumentConflict, ArgumentId};
pub use crate::metadata::Value;
pub use crate::statements::{StatementConflict, StatementId};

/// Stable, source-order id; indexes `Model::pcs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcsId(pub usize);

/// Positional role of a statement within a PCS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Premise,
    IntermediaryConclusion,
    MainConclusion,
}

/// One resolved item in a PCS, in source order. The enum variant carries the
/// item's resolved role/binding. `items` indices are stable for the lifetime
/// of the [`ResolvedPcs`]; consumers must not filter or reorder in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedPcsItem {
    Statement {
        role: Role,
        number: usize,
        /// The Model statement this occurrence resolves to (titled → merged by
        /// title; untitled → its minted singleton). Always resolvable.
        statement: StatementId,
        span: Span,
    },
    Inference {
        rules: Vec<String>,
        /// Absolute index into the parent `items` vector of the conclusion
        /// statement. `None` when degenerate (trailing inference, consecutive
        /// inferences). Relations between an inference and its conclusion are
        /// transparent — the next statement in source order is the conclusion.
        concludes_item_idx: Option<usize>,
        span: Span,
    },
    /// Pass-through: B5 resolves relation targets; B4b preserves position so B5
    /// has structural context without re-walking the AST.
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
/// `i` below B3's count equals B3's `StatementId(i)` entity.
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
    /// The *first* PCS immediately adjacent to this argument, if any (always
    /// `Some` for anonymous arguments). In the rare case a named title is
    /// defined more than once with each definition followed by a PCS (itself an
    /// argument redefinition, recorded in [`Model::argument_conflicts`]), only
    /// the first such PCS is recorded here; every owning edge is authoritative
    /// on [`ResolvedPcs::argument`], so walk `pcs` for the back-reference but
    /// `ResolvedPcs::argument` for the complete argument→PCS relation.
    pub pcs: Option<PcsId>,
}

/// Genuine internal PCS malformations, surfaced as data (never `Result`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PcsIssue {
    /// An inference line has no statement to conclude (trailing, or only a
    /// relation follows it).
    InferenceWithNoConclusion { inference_span: Span },
    /// Two inference lines in a row; the first has no conclusion.
    ConsecutiveInferences { first_span: Span, second_span: Span },
    /// A PCS block with zero items. Defensive: the parser requires a leading
    /// numbered statement to open a PCS, so it never emits an empty one today;
    /// this keeps `build_model` total against any future AST source.
    EmptyPcs { pcs_span: Span },
}

/// A node in the dialectical graph: a statement equivalence class or an
/// argument. Edges connect these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Node {
    Statement(StatementId),
    Argument(ArgumentId),
}

/// The kind of a dialectical edge. `Contradictory` is symmetric but stored as a
/// single directed edge; consumers treat it as symmetric. Undercut is recorded
/// as an edge here; mapping it onto an inference step is B6's job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    Support,
    Attack,
    Undercut,
    Contradictory,
}

/// A resolved directed dialectical edge between two nodes. Deduped on
/// `(from, to, kind)` (the `span` of the first occurrence is kept).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: Node,
    pub to: Node,
    pub kind: RelationKind,
    pub span: Span,
}

/// A relation that could not be resolved, surfaced as data (never `Result`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationIssue {
    /// A relation with no enclosing element to attach to (no less-indented
    /// node precedes it).
    RelationWithoutParent { span: Span },
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
    /// set. Supersets B3's.
    pub statement_conflicts: Vec<StatementConflict>,
    /// Argument redefinition conflicts (carried from B4a).
    pub argument_conflicts: Vec<ArgumentConflict>,
    /// Internal PCS malformations.
    pub issues: Vec<PcsIssue>,
    /// Deduped directed dialectical edges between nodes (by `(from, to, kind)`,
    /// in first-occurrence source order). Top-level and PCS-interspersed
    /// relations both contribute.
    pub edges: Vec<Edge>,
    /// Relations that could not be resolved (e.g. no parent).
    pub relation_issues: Vec<RelationIssue>,
}

/// Build the complete semantic model for a parsed document.
///
/// Reuses [`crate::build_statements`] and [`crate::build_arguments`] for the
/// seed registries (prefix-correspondence), then resolves every `Block::Pcs`:
/// links it to its argument by strict adjacency (minting anonymous arguments
/// for standalone PCSs), resolves each PCS statement to the unified registry
/// (titled merged by title; untitled minted as singletons), assigns positional
/// roles, and binds each inference to its conclusion. Total — malformations and
/// conflicts are data.
pub fn build_model(document: &Document) -> Model {
    // Phase 1: seed the registries by reusing B3/B4a (prefix-correspondence).
    let statements = crate::build_statements(document);
    let arguments = crate::build_arguments(document);

    let mut model_statements: Vec<ModelStatement> = statements
        .statements
        .iter()
        .map(|s| ModelStatement {
            id: s.id,
            title: Some(s.title.clone()),
            canonical_text: s.canonical_text.clone(),
            canonical_metadata: s.canonical_metadata.clone(),
        })
        .collect();
    let mut by_title_stmt: HashMap<String, StatementId> = model_statements
        .iter()
        .filter_map(|s| s.title.clone().map(|t| (t, s.id)))
        .collect();

    let mut model_arguments: Vec<ModelArgument> = arguments
        .arguments
        .iter()
        .map(|a| ModelArgument {
            id: a.id,
            title: Some(a.title.clone()),
            canonical_description: a.canonical_description.clone(),
            canonical_metadata: a.canonical_metadata.clone(),
            pcs: None,
        })
        .collect();
    let mut by_title_arg: HashMap<String, ArgumentId> = model_arguments
        .iter()
        .filter_map(|a| a.title.clone().map(|t| (t, a.id)))
        .collect();

    // Conflicts seeded from B3/B4a; statement conflicts extended with PCS
    // redefinitions below. `conflict_idx` finds an existing per-title conflict
    // to append to; `canonical_spans` holds each title's first-definition span.
    let mut statement_conflicts = statements.conflicts.clone();
    let argument_conflicts = arguments.conflicts.clone();
    let mut conflict_idx: HashMap<String, usize> = statement_conflicts
        .iter()
        .enumerate()
        .map(|(i, c)| (c.title.clone(), i))
        .collect();
    let mut canonical_spans: HashMap<String, Span> = HashMap::new();
    for block in &document.blocks {
        if let Block::Statement(s) = block
            && let Some(title) = &s.title
            && !s.is_reference
        {
            canonical_spans.entry(title.clone()).or_insert(s.span);
        }
    }

    // Phase 2: walk blocks — PCS resolution + dialectical edge resolution.
    let mut pcs: Vec<ResolvedPcs> = Vec::new();
    let mut block_pcs: Vec<Option<PcsId>> = Vec::with_capacity(document.blocks.len());
    let mut issues: Vec<PcsIssue> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut relation_issues: Vec<RelationIssue> = Vec::new();
    // Indent stack of the current node at each open level. A relation attaches
    // to the nearest strictly-less-indented frame. Content anchors (statements,
    // arguments, PCS statements — none of which carry an indent in the AST) sit
    // at `ANCHOR_INDENT` (below any relation indent), so even an unindented
    // relation (`indent 0`, which our parser accepts though the reference does
    // not) still attaches to the statement above it.
    let mut stack: Vec<Frame> = Vec::new();

    for (i, block) in document.blocks.iter().enumerate() {
        match block {
            Block::Heading(_) => {
                // A heading starts a new structural context for relations.
                stack.clear();
                block_pcs.push(None);
            }
            Block::Statement(s) => {
                let node = top_level_statement_node(
                    s,
                    &mut model_statements,
                    &mut by_title_stmt,
                    &mut statement_conflicts,
                    &mut conflict_idx,
                    &mut canonical_spans,
                );
                enter(&mut stack, ANCHOR_INDENT, node);
                block_pcs.push(None);
            }
            Block::Argument(a) => {
                if let Some(&id) = by_title_arg.get(&a.title) {
                    enter(&mut stack, ANCHOR_INDENT, Node::Argument(id));
                }
                block_pcs.push(None);
            }
            Block::Relation(r) => {
                resolve_relation(
                    r,
                    &mut stack,
                    &mut model_statements,
                    &mut by_title_stmt,
                    &mut model_arguments,
                    &mut by_title_arg,
                    &mut statement_conflicts,
                    &mut conflict_idx,
                    &mut canonical_spans,
                    &mut edges,
                    &mut relation_issues,
                );
                block_pcs.push(None);
            }
            Block::Pcs(p) => {
                let pcs_id = PcsId(pcs.len());

                // Linkage: strict adjacency. A PCS attaches to a named argument
                // only when that argument block is the immediately-preceding
                // block; otherwise it is its own anonymous argument.
                let owner = match i.checked_sub(1).map(|prev| &document.blocks[prev]) {
                    Some(Block::Argument(a)) => match by_title_arg.get(&a.title) {
                        Some(&id) => {
                            if model_arguments[id.0].pcs.is_none() {
                                model_arguments[id.0].pcs = Some(pcs_id);
                            }
                            id
                        }
                        None => mint_anonymous_argument(&mut model_arguments, pcs_id),
                    },
                    _ => mint_anonymous_argument(&mut model_arguments, pcs_id),
                };

                let items = resolve_pcs_items(
                    p,
                    &mut model_statements,
                    &mut by_title_stmt,
                    &mut statement_conflicts,
                    &mut conflict_idx,
                    &mut canonical_spans,
                    &mut issues,
                );

                // Interspersed PCS relations resolve in a PCS-local scope (their
                // sources are the PCS's own statements, not the top-level stack).
                resolve_pcs_relations(
                    &items,
                    &mut model_statements,
                    &mut by_title_stmt,
                    &mut model_arguments,
                    &mut by_title_arg,
                    &mut statement_conflicts,
                    &mut conflict_idx,
                    &mut canonical_spans,
                    &mut edges,
                    &mut relation_issues,
                );

                pcs.push(ResolvedPcs {
                    id: pcs_id,
                    argument: owner,
                    items,
                    span: p.span,
                });
                block_pcs.push(Some(pcs_id));
            }
        }
    }

    Model {
        statements: model_statements,
        arguments: model_arguments,
        pcs,
        block_pcs,
        statement_conflicts,
        argument_conflicts,
        issues,
        edges,
        relation_issues,
    }
}

/// The stack indent of a content anchor (statement / argument / PCS statement).
/// Below any relation indent (which is `>= 0`), so an unindented relation still
/// attaches to the content above it.
const ANCHOR_INDENT: isize = -1;

/// A node on the indent stack: the current node at a given indentation level.
#[derive(Debug, Clone, Copy)]
struct Frame {
    indent: isize,
    node: Node,
}

/// Push `node` at `indent`, first popping any frames at the same or deeper
/// indentation (monotonic discipline).
fn enter(stack: &mut Vec<Frame>, indent: isize, node: Node) {
    while stack.last().is_some_and(|f| f.indent >= indent) {
        stack.pop();
    }
    stack.push(Frame { indent, node });
}

/// The node for a top-level statement: a titled one is already in the registry
/// (from B3); a plain one is minted as a singleton so it can anchor relations.
fn top_level_statement_node(
    s: &Statement,
    model_statements: &mut Vec<ModelStatement>,
    by_title: &mut HashMap<String, StatementId>,
    conflicts: &mut Vec<StatementConflict>,
    conflict_idx: &mut HashMap<String, usize>,
    canonical_spans: &mut HashMap<String, Span>,
) -> Node {
    if let Some(t) = &s.title
        && let Some(&id) = by_title.get(t)
    {
        return Node::Statement(id);
    }
    Node::Statement(resolve_pcs_statement(
        s,
        model_statements,
        by_title,
        conflicts,
        conflict_idx,
        canonical_spans,
    ))
}

/// Resolve a top-level relation against the indent stack: peek the parent
/// (nearest strictly-less-indented frame), resolve the target to a node, emit
/// the directed edge, then push the target so deeper relations nest under it.
#[allow(clippy::too_many_arguments)]
fn resolve_relation(
    r: &Relation,
    stack: &mut Vec<Frame>,
    model_statements: &mut Vec<ModelStatement>,
    by_title_stmt: &mut HashMap<String, StatementId>,
    model_arguments: &mut Vec<ModelArgument>,
    by_title_arg: &mut HashMap<String, ArgumentId>,
    conflicts: &mut Vec<StatementConflict>,
    conflict_idx: &mut HashMap<String, usize>,
    canonical_spans: &mut HashMap<String, Span>,
    edges: &mut Vec<Edge>,
    relation_issues: &mut Vec<RelationIssue>,
) {
    let indent = r.indent as isize;
    let Some(parent) = stack
        .iter()
        .rev()
        .find(|f| f.indent < indent)
        .map(|f| f.node)
    else {
        relation_issues.push(RelationIssue::RelationWithoutParent { span: r.span });
        return;
    };
    let target = resolve_relation_target(
        &r.target,
        model_statements,
        by_title_stmt,
        model_arguments,
        by_title_arg,
        conflicts,
        conflict_idx,
        canonical_spans,
    );
    let (from, to) = orient(parent, target, &r.direction);
    push_edge_if_new(edges, from, to, relation_kind(&r.operator), r.span);
    enter(stack, indent, target);
}

/// Resolve the interspersed relations of one PCS in a local scope: each PCS
/// statement is the base node for the relations that follow it; nested relations
/// attach to the enclosing relation's target.
#[allow(clippy::too_many_arguments)]
fn resolve_pcs_relations(
    items: &[ResolvedPcsItem],
    model_statements: &mut Vec<ModelStatement>,
    by_title_stmt: &mut HashMap<String, StatementId>,
    model_arguments: &mut Vec<ModelArgument>,
    by_title_arg: &mut HashMap<String, ArgumentId>,
    conflicts: &mut Vec<StatementConflict>,
    conflict_idx: &mut HashMap<String, usize>,
    canonical_spans: &mut HashMap<String, Span>,
    edges: &mut Vec<Edge>,
    relation_issues: &mut Vec<RelationIssue>,
) {
    let mut stack: Vec<Frame> = Vec::new();
    for item in items {
        match item {
            ResolvedPcsItem::Statement { statement, .. } => {
                enter(&mut stack, ANCHOR_INDENT, Node::Statement(*statement));
            }
            ResolvedPcsItem::Relation(r) => {
                let indent = r.indent as isize;
                let Some(parent) = stack
                    .iter()
                    .rev()
                    .find(|f| f.indent < indent)
                    .map(|f| f.node)
                else {
                    relation_issues.push(RelationIssue::RelationWithoutParent { span: r.span });
                    continue;
                };
                let target = resolve_relation_target(
                    &r.target,
                    model_statements,
                    by_title_stmt,
                    model_arguments,
                    by_title_arg,
                    conflicts,
                    conflict_idx,
                    canonical_spans,
                );
                let (from, to) = orient(parent, target, &r.direction);
                push_edge_if_new(edges, from, to, relation_kind(&r.operator), r.span);
                enter(&mut stack, indent, target);
            }
            ResolvedPcsItem::Inference { .. } => {}
        }
    }
}

/// Resolve a relation target to a node, minting/merging into the registry.
#[allow(clippy::too_many_arguments)]
fn resolve_relation_target(
    target: &RelationTarget,
    model_statements: &mut Vec<ModelStatement>,
    by_title_stmt: &mut HashMap<String, StatementId>,
    model_arguments: &mut Vec<ModelArgument>,
    by_title_arg: &mut HashMap<String, ArgumentId>,
    conflicts: &mut Vec<StatementConflict>,
    conflict_idx: &mut HashMap<String, usize>,
    canonical_spans: &mut HashMap<String, Span>,
) -> Node {
    match target {
        RelationTarget::Statement(s) => Node::Statement(resolve_pcs_statement(
            s,
            model_statements,
            by_title_stmt,
            conflicts,
            conflict_idx,
            canonical_spans,
        )),
        RelationTarget::Argument(a) => {
            Node::Argument(resolve_argument_node(a, model_arguments, by_title_arg))
        }
    }
}

/// Resolve a relation-target argument by title: merge with an existing argument
/// (filling its canonical description on first definition) or mint a new one.
///
/// Asymmetry with statements (deferred): a relation-target argument *re*definition
/// (`<A>: d1` then `- <A>: d2`) keeps the first description but does not record an
/// `ArgumentConflict` — arguments carry no canonical-span bookkeeping here, and
/// argument redefinition is far rarer than statement redefinition. Top-level
/// argument conflicts are still captured by B4a.
fn resolve_argument_node(
    a: &argdown_core::Argument,
    model_arguments: &mut Vec<ModelArgument>,
    by_title_arg: &mut HashMap<String, ArgumentId>,
) -> ArgumentId {
    if let Some(&id) = by_title_arg.get(&a.title) {
        if !a.is_reference && model_arguments[id.0].canonical_description.is_none() {
            model_arguments[id.0].canonical_description = Some(a.description.clone());
            model_arguments[id.0].canonical_metadata = a
                .metadata
                .as_ref()
                .map(crate::metadata::parse_metadata)
                .transpose()
                .ok()
                .flatten();
        }
        return id;
    }
    let id = ArgumentId(model_arguments.len());
    let defined = !a.is_reference;
    model_arguments.push(ModelArgument {
        id,
        title: Some(a.title.clone()),
        canonical_description: defined.then(|| a.description.clone()),
        canonical_metadata: defined
            .then(|| {
                a.metadata
                    .as_ref()
                    .map(crate::metadata::parse_metadata)
                    .transpose()
                    .ok()
                    .flatten()
            })
            .flatten(),
        pcs: None,
    });
    by_title_arg.insert(a.title.clone(), id);
    id
}

/// Orient a relation into a directed `(from, to)` edge per its direction.
fn orient(parent: Node, target: Node, direction: &RelationDirection) -> (Node, Node) {
    match direction {
        RelationDirection::Inbound => (target, parent),
        RelationDirection::Outbound | RelationDirection::Bidirectional => (parent, target),
    }
}

/// Map the parser's `RelationOperator` to a `RelationKind`.
fn relation_kind(operator: &RelationOperator) -> RelationKind {
    match operator {
        RelationOperator::Support => RelationKind::Support,
        RelationOperator::Attack => RelationKind::Attack,
        RelationOperator::Undercut => RelationKind::Undercut,
        RelationOperator::Contradictory => RelationKind::Contradictory,
    }
}

/// Append an edge unless one with the same `(from, to, kind)` already exists
/// (the first occurrence's span is kept). Linear scan — O(n) per insert, O(n²)
/// over a document; fine at expected document sizes. If large dialectical maps
/// ever matter, pair `edges` with a `HashSet<(Node, Node, RelationKind)>`.
fn push_edge_if_new(edges: &mut Vec<Edge>, from: Node, to: Node, kind: RelationKind, span: Span) {
    if !edges
        .iter()
        .any(|e| e.from == from && e.to == to && e.kind == kind)
    {
        edges.push(Edge {
            from,
            to,
            kind,
            span,
        });
    }
}

/// Append a fresh anonymous argument (a standalone PCS) and return its id.
fn mint_anonymous_argument(arguments: &mut Vec<ModelArgument>, pcs: PcsId) -> ArgumentId {
    let id = ArgumentId(arguments.len());
    arguments.push(ModelArgument {
        id,
        title: None,
        canonical_description: None,
        canonical_metadata: None,
        pcs: Some(pcs),
    });
    id
}

/// Resolve one PCS into roled items: bind inferences to the conclusion that
/// follows them (relations transparent), resolve every statement to the unified
/// registry, finalize positional roles, and emit issues for degenerate cases.
#[allow(clippy::too_many_arguments)]
fn resolve_pcs_items(
    p: &argdown_core::Pcs,
    model_statements: &mut Vec<ModelStatement>,
    by_title: &mut HashMap<String, StatementId>,
    conflicts: &mut Vec<StatementConflict>,
    conflict_idx: &mut HashMap<String, usize>,
    canonical_spans: &mut HashMap<String, Span>,
    issues: &mut Vec<PcsIssue>,
) -> Vec<ResolvedPcsItem> {
    if p.items.is_empty() {
        issues.push(PcsIssue::EmptyPcs { pcs_span: p.span });
    }

    let mut items: Vec<ResolvedPcsItem> = Vec::with_capacity(p.items.len());
    // The `items` index of an inference still awaiting its conclusion.
    let mut pending_inference: Option<usize> = None;

    for item in &p.items {
        match item {
            PcsItem::Statement {
                number,
                statement,
                span,
            } => {
                let id = resolve_pcs_statement(
                    statement,
                    model_statements,
                    by_title,
                    conflicts,
                    conflict_idx,
                    canonical_spans,
                );
                // Bind a pending inference to this statement (its conclusion).
                let role = if let Some(inf_idx) = pending_inference.take() {
                    let conclusion_idx = items.len();
                    if let ResolvedPcsItem::Inference {
                        concludes_item_idx, ..
                    } = &mut items[inf_idx]
                    {
                        *concludes_item_idx = Some(conclusion_idx);
                    }
                    // Provisional; finalized after the loop.
                    Role::IntermediaryConclusion
                } else {
                    Role::Premise
                };
                items.push(ResolvedPcsItem::Statement {
                    role,
                    number: *number,
                    statement: id,
                    span: *span,
                });
            }
            PcsItem::Inference { rules, span, .. } => {
                // `pending_inference` only ever holds the index of an
                // `Inference` item, so the let-chain always matches.
                if let Some(prev) = pending_inference.take()
                    && let ResolvedPcsItem::Inference {
                        span: first_span, ..
                    } = &items[prev]
                {
                    issues.push(PcsIssue::ConsecutiveInferences {
                        first_span: *first_span,
                        second_span: *span,
                    });
                }
                items.push(ResolvedPcsItem::Inference {
                    rules: rules.clone(),
                    concludes_item_idx: None,
                    span: *span,
                });
                pending_inference = Some(items.len() - 1);
            }
            PcsItem::Relation(r) => {
                // Transparent: a relation does not break inference binding.
                items.push(ResolvedPcsItem::Relation(r.clone()));
            }
        }
    }

    // A still-pending inference at the end has no conclusion.
    if let Some(inf_idx) = pending_inference.take()
        && let ResolvedPcsItem::Inference { span, .. } = &items[inf_idx]
    {
        issues.push(PcsIssue::InferenceWithNoConclusion {
            inference_span: *span,
        });
    }

    finalize_roles(&mut items);
    items
}

/// Among the bound conclusions, the last in item order is the main conclusion;
/// earlier ones are intermediary. Unbound statements stay premises.
fn finalize_roles(items: &mut [ResolvedPcsItem]) {
    let conclusions: Vec<usize> = items
        .iter()
        .filter_map(|it| match it {
            ResolvedPcsItem::Inference {
                concludes_item_idx: Some(ci),
                ..
            } => Some(*ci),
            _ => None,
        })
        .collect();
    let Some(&main) = conclusions.iter().max() else {
        return;
    };
    for &ci in &conclusions {
        if let ResolvedPcsItem::Statement { role, .. } = &mut items[ci] {
            *role = if ci == main {
                Role::MainConclusion
            } else {
                Role::IntermediaryConclusion
            };
        }
    }
}

/// Resolve a PCS statement to the unified registry: titled statements merge by
/// title (filling the canonical on first definition, recording a conflict on a
/// later one); untitled statements are minted as singletons.
fn resolve_pcs_statement(
    statement: &Statement,
    model_statements: &mut Vec<ModelStatement>,
    by_title: &mut HashMap<String, StatementId>,
    conflicts: &mut Vec<StatementConflict>,
    conflict_idx: &mut HashMap<String, usize>,
    canonical_spans: &mut HashMap<String, Span>,
) -> StatementId {
    let Some(title) = &statement.title else {
        // Untitled: a singleton equivalence class (always a definition).
        let id = StatementId(model_statements.len());
        model_statements.push(ModelStatement {
            id,
            title: None,
            canonical_text: Some(statement.text.clone()),
            canonical_metadata: parse_canonical_metadata(statement),
        });
        return id;
    };

    let id = *by_title.entry(title.clone()).or_insert_with(|| {
        let id = StatementId(model_statements.len());
        model_statements.push(ModelStatement {
            id,
            title: Some(title.clone()),
            canonical_text: None,
            canonical_metadata: None,
        });
        id
    });

    if !statement.is_reference {
        if model_statements[id.0].canonical_text.is_none() {
            model_statements[id.0].canonical_text = Some(statement.text.clone());
            model_statements[id.0].canonical_metadata = parse_canonical_metadata(statement);
            canonical_spans
                .entry(title.clone())
                .or_insert(statement.span);
        } else {
            // Redefinition over the unified (top-level + PCS) definition set.
            let canonical_span = canonical_spans
                .get(title)
                .copied()
                .unwrap_or(statement.span);
            match conflict_idx.get(title) {
                Some(&ci) => conflicts[ci].conflicting_spans.push(statement.span),
                None => {
                    let ci = conflicts.len();
                    conflicts.push(StatementConflict {
                        title: title.clone(),
                        canonical_span,
                        conflicting_spans: vec![statement.span],
                    });
                    conflict_idx.insert(title.clone(), ci);
                }
            }
        }
    }
    id
}

/// Parse a statement's metadata via B2, absorbing parse errors as `None`
/// (the model is total).
fn parse_canonical_metadata(statement: &Statement) -> Option<Value> {
    statement
        .metadata
        .as_ref()
        .map(crate::metadata::parse_metadata)
        .transpose()
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_parser::parse;

    /// Helper: the `StatementId` of a `ResolvedPcsItem::Statement`.
    fn stmt_of(item: &ResolvedPcsItem) -> StatementId {
        match item {
            ResolvedPcsItem::Statement { statement, .. } => *statement,
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    fn role_of(item: &ResolvedPcsItem) -> Role {
        match item {
            ResolvedPcsItem::Statement { role, .. } => *role,
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn single_step_pcs_attached_to_named_argument() {
        let doc = parse("<A>: d\n\n(1) P1\n----\n(2) C1").unwrap();
        let m = build_model(&doc);

        // One named argument, owning the one PCS.
        assert_eq!(m.arguments.len(), 1);
        assert_eq!(m.arguments[0].title.as_deref(), Some("A"));
        assert_eq!(m.arguments[0].pcs, Some(PcsId(0)));

        // One resolved PCS, owned by A.
        assert_eq!(m.pcs.len(), 1);
        let pcs = &m.pcs[0];
        assert_eq!(pcs.argument, ArgumentId(0));
        assert_eq!(pcs.items.len(), 3);

        // Roles: premise, inference, main-conclusion.
        assert_eq!(role_of(&pcs.items[0]), Role::Premise);
        assert!(matches!(
            &pcs.items[1],
            ResolvedPcsItem::Inference {
                concludes_item_idx: Some(2),
                ..
            }
        ));
        assert_eq!(role_of(&pcs.items[2]), Role::MainConclusion);

        // Two untitled singleton statements.
        assert_eq!(m.statements.len(), 2);
        assert!(m.statements.iter().all(|s| s.title.is_none()));
        assert_eq!(stmt_of(&pcs.items[0]), StatementId(0));
        assert_eq!(stmt_of(&pcs.items[2]), StatementId(1));

        // block_pcs index-aligned: [Argument, Pcs].
        assert_eq!(m.block_pcs, vec![None, Some(PcsId(0))]);
        assert!(m.issues.is_empty());
        assert!(m.statement_conflicts.is_empty());
    }

    #[test]
    fn empty_document_has_empty_model() {
        let m = build_model(&parse("").unwrap());
        assert_eq!(m, Model::default());
    }

    // ── Linkage (reference-faithful: strict adjacency) ──────────────────────

    #[test]
    fn bare_reference_argument_attaches_pcs() {
        let m = build_model(&parse("<A>\n\n(1) P1\n----\n(2) C1").unwrap());
        assert_eq!(m.arguments.len(), 1);
        assert_eq!(m.arguments[0].title.as_deref(), Some("A"));
        assert_eq!(m.arguments[0].pcs, Some(PcsId(0)));
        assert_eq!(m.pcs[0].argument, ArgumentId(0));
    }

    #[test]
    fn section_between_detaches_pcs_to_anonymous_argument() {
        let m = build_model(&parse("<A>: d\n\n# S\n\n(1) P1\n----\n(2) C1").unwrap());
        assert_eq!(m.arguments.len(), 2);
        // Named A keeps no PCS; the standalone PCS is its own anonymous argument.
        assert_eq!(m.arguments[0].title.as_deref(), Some("A"));
        assert_eq!(m.arguments[0].pcs, None);
        assert_eq!(m.arguments[1].title, None);
        assert_eq!(m.arguments[1].pcs, Some(PcsId(0)));
        assert_eq!(m.pcs[0].argument, ArgumentId(1));
    }

    #[test]
    fn statement_between_detaches_pcs_to_anonymous_argument() {
        let m = build_model(&parse("<A>: d\n\n[S]: text\n\n(1) P1\n----\n(2) C1").unwrap());
        assert_eq!(m.arguments.len(), 2);
        assert_eq!(m.arguments[0].pcs, None);
        assert_eq!(m.arguments[1].title, None);
        assert_eq!(m.pcs[0].argument, ArgumentId(1));
    }

    #[test]
    fn second_pcs_after_one_argument_is_anonymous() {
        let m =
            build_model(&parse("<A>: d\n\n(1) P1\n----\n(2) C1\n\n(1) Q1\n----\n(2) C2").unwrap());
        assert_eq!(m.pcs.len(), 2);
        // First PCS attaches to A; second is its own anonymous argument.
        assert_eq!(m.pcs[0].argument, ArgumentId(0));
        assert_eq!(m.arguments[0].title.as_deref(), Some("A"));
        assert_eq!(m.arguments[0].pcs, Some(PcsId(0)));
        assert_eq!(m.pcs[1].argument, ArgumentId(1));
        assert_eq!(m.arguments[1].title, None);
        assert_eq!(m.arguments[1].pcs, Some(PcsId(1)));
    }

    #[test]
    fn pcs_with_no_preceding_argument_is_anonymous() {
        let m = build_model(&parse("(1) P1\n----\n(2) C1").unwrap());
        assert_eq!(m.arguments.len(), 1);
        assert_eq!(m.arguments[0].title, None);
        assert_eq!(m.arguments[0].pcs, Some(PcsId(0)));
        assert_eq!(m.pcs[0].argument, ArgumentId(0));
        assert_eq!(m.block_pcs, vec![Some(PcsId(0))]);
    }

    // ── Roles + inference ───────────────────────────────────────────────────

    #[test]
    fn two_step_pcs_assigns_intermediary_and_main() {
        let m = build_model(
            &parse("<A>: d\n\n(1) P1\n(2) P2\n-- R --\n(3) Inter\n(4) P3\n----\n(5) Main").unwrap(),
        );
        let items = &m.pcs[0].items;
        // 0=P1 1=P2 2=Inf(R) 3=Inter 4=P3 5=Inf 6=Main
        assert_eq!(items.len(), 7);
        assert_eq!(role_of(&items[0]), Role::Premise);
        assert_eq!(role_of(&items[1]), Role::Premise);
        assert_eq!(role_of(&items[3]), Role::IntermediaryConclusion);
        assert_eq!(role_of(&items[4]), Role::Premise);
        assert_eq!(role_of(&items[6]), Role::MainConclusion);
        assert!(matches!(
            &items[2],
            ResolvedPcsItem::Inference { rules, concludes_item_idx: Some(3), .. } if rules == &["R".to_string()]
        ));
        assert!(matches!(
            &items[5],
            ResolvedPcsItem::Inference { rules, concludes_item_idx: Some(6), .. } if rules.is_empty()
        ));
    }

    #[test]
    fn premises_only_pcs_has_no_conclusion() {
        let m = build_model(&parse("<A>: d\n\n(1) P1\n(2) P2").unwrap());
        let items = &m.pcs[0].items;
        assert_eq!(items.len(), 2);
        assert_eq!(role_of(&items[0]), Role::Premise);
        assert_eq!(role_of(&items[1]), Role::Premise);
        assert!(m.issues.is_empty());
    }

    #[test]
    fn ruled_inference_captures_multiple_rule_names() {
        let m = build_model(&parse("<A>: d\n\n(1) P\n-- Rule A, Rule B --\n(2) C").unwrap());
        assert!(matches!(
            &m.pcs[0].items[1],
            ResolvedPcsItem::Inference { rules, .. }
                if rules == &["Rule A".to_string(), "Rule B".to_string()]
        ));
    }

    // ── Statement resolution + unified registry ─────────────────────────────

    #[test]
    fn titled_pcs_statement_merges_with_top_level_same_title() {
        let m = build_model(&parse("[P]: top\n\n<A>: d\n\n(1) [P]\n----\n(2) C").unwrap());
        // P is the same equivalence class top-level and in the PCS.
        let p_id = stmt_of(&m.pcs[0].items[0]);
        assert_eq!(p_id, StatementId(0));
        assert_eq!(m.statements[0].title.as_deref(), Some("P"));
        assert_eq!(m.statements[0].canonical_text.as_deref(), Some("top"));
        // statements: P (titled) + C (untitled singleton).
        assert_eq!(m.statements.len(), 2);
    }

    #[test]
    fn pcs_only_titled_statement_gets_its_own_class() {
        let m = build_model(&parse("<A>: d\n\n(1) [P]: text\n----\n(2) C").unwrap());
        assert_eq!(m.statements[0].title.as_deref(), Some("P"));
        assert_eq!(m.statements[0].canonical_text.as_deref(), Some("text"));
    }

    #[test]
    fn untitled_pcs_statements_are_distinct_singletons() {
        let m = build_model(&parse("<A>: d\n\n(1) same\n----\n(2) same").unwrap());
        assert_eq!(m.statements.len(), 2);
        assert!(m.statements.iter().all(|s| s.title.is_none()));
        assert_ne!(stmt_of(&m.pcs[0].items[0]), stmt_of(&m.pcs[0].items[2]));
    }

    #[test]
    fn top_level_then_pcs_redefinition_records_a_conflict() {
        let m =
            build_model(&parse("[P]: first\n\n<A>: d\n\n(1) [P]: second\n----\n(2) C").unwrap());
        assert_eq!(m.statements[0].canonical_text.as_deref(), Some("first"));
        assert_eq!(m.statement_conflicts.len(), 1);
        assert_eq!(m.statement_conflicts[0].title, "P");
        assert_eq!(m.statement_conflicts[0].conflicting_spans.len(), 1);
    }

    #[test]
    fn canonical_metadata_is_parsed_for_pcs_statement() {
        let m = build_model(&parse("<A>: d\n\n(1) [P]: text { key: value }\n----\n(2) C").unwrap());
        let meta = m.statements[0]
            .canonical_metadata
            .as_ref()
            .expect("P had metadata");
        assert!(matches!(meta, Value::Mapping(map) if map.contains_key("key")));
    }

    // ── Degenerate cases (issues) ───────────────────────────────────────────

    #[test]
    fn interspersed_relation_does_not_break_binding() {
        // `  +> [S]` is a child relation of premise (1); the inference still
        // binds to the conclusion that follows.
        let m = build_model(&parse("<A>: d\n\n(1) P\n  +> [S]\n----\n(2) C").unwrap());
        let items = &m.pcs[0].items;
        // 0=P 1=Relation 2=Inf 3=C
        assert_eq!(items.len(), 4);
        assert!(matches!(&items[1], ResolvedPcsItem::Relation(_)));
        assert!(matches!(
            &items[2],
            ResolvedPcsItem::Inference {
                concludes_item_idx: Some(3),
                ..
            }
        ));
        assert_eq!(role_of(&items[3]), Role::MainConclusion);
        assert!(m.issues.is_empty());
    }

    #[test]
    fn pcs_only_redefinition_across_two_pcs_records_a_conflict() {
        // [P] is defined in the first PCS and redefined in a second; neither is
        // top-level. Exercises the canonical-span insert-then-lookup path.
        let m = build_model(&parse("(1) [P]: a\n----\n(2) x\n\n(1) [P]: b\n----\n(2) y").unwrap());
        let p = m
            .statements
            .iter()
            .find(|s| s.title.as_deref() == Some("P"))
            .expect("P is registered");
        assert_eq!(p.canonical_text.as_deref(), Some("a"));
        assert_eq!(m.statement_conflicts.len(), 1);
        assert_eq!(m.statement_conflicts[0].title, "P");
        assert_eq!(m.statement_conflicts[0].conflicting_spans.len(), 1);
    }

    #[test]
    fn argument_defined_twice_each_with_a_pcs_keeps_one_back_pointer() {
        // Both `<A>` definitions merge to one argument (an argument conflict);
        // both PCSs own it via `ResolvedPcs::argument`, but `pcs` is first-wins.
        let m = build_model(
            &parse("<A>: d1\n\n(1) P\n----\n(2) C\n\n<A>: d2\n\n(1) Q\n----\n(2) D").unwrap(),
        );
        assert_eq!(m.pcs.len(), 2);
        assert_eq!(m.pcs[0].argument, ArgumentId(0));
        assert_eq!(m.pcs[1].argument, ArgumentId(0));
        assert_eq!(m.arguments[0].pcs, Some(PcsId(0)));
        assert_eq!(m.argument_conflicts.len(), 1);
        assert_eq!(m.argument_conflicts[0].title, "A");
    }

    #[test]
    fn titled_intermediary_conclusion_merges_with_top_level_statement() {
        // (2) [I] is both an intermediary conclusion and a reference to the
        // top-level [I] — role assignment and by-title merge must agree.
        let m = build_model(
            &parse("[I]: top\n\n<A>: d\n\n(1) P\n----\n(2) [I]\n(3) Q\n----\n(4) Main").unwrap(),
        );
        let items = &m.pcs[0].items;
        // 0=P 1=Inf 2=[I] 3=Q 4=Inf 5=Main
        assert_eq!(role_of(&items[2]), Role::IntermediaryConclusion);
        assert_eq!(stmt_of(&items[2]), StatementId(0));
        assert_eq!(m.statements[0].title.as_deref(), Some("I"));
        assert_eq!(role_of(&items[5]), Role::MainConclusion);
    }

    // ── Index alignment + prefix-correspondence ─────────────────────────────

    #[test]
    fn trailing_inference_reports_an_issue() {
        let m = build_model(&parse("<A>: d\n\n(1) P\n----").unwrap());
        let items = &m.pcs[0].items;
        assert!(matches!(
            &items[1],
            ResolvedPcsItem::Inference {
                concludes_item_idx: None,
                ..
            }
        ));
        assert!(
            m.issues
                .iter()
                .any(|i| matches!(i, PcsIssue::InferenceWithNoConclusion { .. }))
        );
        assert_eq!(role_of(&items[0]), Role::Premise);
    }

    #[test]
    fn consecutive_inferences_report_an_issue() {
        // 0=P 1=Inf 2=Inf 3=C — the first inference has no conclusion.
        let m = build_model(&parse("<A>: d\n\n(1) P\n----\n----\n(2) C").unwrap());
        let items = &m.pcs[0].items;
        assert!(matches!(
            &items[1],
            ResolvedPcsItem::Inference {
                concludes_item_idx: None,
                ..
            }
        ));
        assert!(matches!(
            &items[2],
            ResolvedPcsItem::Inference {
                concludes_item_idx: Some(3),
                ..
            }
        ));
        assert_eq!(role_of(&items[3]), Role::MainConclusion);
        let issue = m
            .issues
            .iter()
            .find_map(|i| match i {
                PcsIssue::ConsecutiveInferences {
                    first_span,
                    second_span,
                } => Some((*first_span, *second_span)),
                _ => None,
            })
            .expect("a ConsecutiveInferences issue");
        // The two inference lines have distinct spans (first precedes second).
        assert!(issue.0.start < issue.1.start);
    }

    #[test]
    fn block_pcs_is_index_aligned_with_blocks() {
        let doc = parse("# H\n\n[S]: s\n\n<A>: d\n\n(1) P\n----\n(2) C").unwrap();
        let m = build_model(&doc);
        assert_eq!(m.block_pcs.len(), doc.blocks.len());
        // Only the final PCS block is Some.
        assert_eq!(m.block_pcs.iter().filter(|b| b.is_some()).count(), 1);
        assert_eq!(*m.block_pcs.last().unwrap(), Some(PcsId(0)));
    }

    // ── B5: relations / edges ───────────────────────────────────────────────

    fn stmt_node(m: &Model, title: &str) -> Node {
        let id = m
            .statements
            .iter()
            .find(|s| s.title.as_deref() == Some(title))
            .expect("statement with that title")
            .id;
        Node::Statement(id)
    }

    fn arg_node(m: &Model, title: &str) -> Node {
        let id = m
            .arguments
            .iter()
            .find(|a| a.title.as_deref() == Some(title))
            .expect("argument with that title")
            .id;
        Node::Argument(id)
    }

    fn has_edge(m: &Model, from: Node, to: Node, kind: RelationKind) -> bool {
        m.edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.kind == kind)
    }

    #[test]
    fn inbound_support_points_target_to_parent() {
        let m = build_model(&parse("[A]: a\n  + [B]: b").unwrap());
        assert_eq!(m.edges.len(), 1);
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
    }

    #[test]
    fn outbound_support_points_parent_to_target() {
        let m = build_model(&parse("[A]: a\n  +> [B]: b").unwrap());
        assert!(has_edge(
            &m,
            stmt_node(&m, "A"),
            stmt_node(&m, "B"),
            RelationKind::Support
        ));
    }

    #[test]
    fn inbound_attack_points_target_to_parent() {
        let m = build_model(&parse("[A]: a\n  - [B]: b").unwrap());
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Attack
        ));
    }

    #[test]
    fn contradictory_is_one_directed_edge() {
        let m = build_model(&parse("[A]: a\n  >< [B]: b").unwrap());
        assert_eq!(m.edges.len(), 1);
        assert!(has_edge(
            &m,
            stmt_node(&m, "A"),
            stmt_node(&m, "B"),
            RelationKind::Contradictory
        ));
    }

    #[test]
    fn nested_relation_source_is_the_enclosing_target() {
        let m = build_model(&parse("[A]: a\n  + [B]: b\n    - [C]: c").unwrap());
        assert_eq!(m.edges.len(), 2);
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
        // The nested attack's source is B (the enclosing relation's target), not A.
        assert!(has_edge(
            &m,
            stmt_node(&m, "C"),
            stmt_node(&m, "B"),
            RelationKind::Attack
        ));
    }

    #[test]
    fn plain_relation_target_becomes_a_singleton_node() {
        let m = build_model(&parse("[A]: a\n  + plain reason").unwrap());
        // One untitled singleton (the plain target) plus titled A.
        let plain = m
            .statements
            .iter()
            .find(|s| s.title.is_none() && s.canonical_text.as_deref() == Some("plain reason"))
            .expect("plain target minted as a node");
        assert!(has_edge(
            &m,
            Node::Statement(plain.id),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
    }

    #[test]
    fn argument_relation_target_is_an_argument_node() {
        let m = build_model(&parse("[A]: a\n  - <Arg>: an arg").unwrap());
        assert!(has_edge(
            &m,
            arg_node(&m, "Arg"),
            stmt_node(&m, "A"),
            RelationKind::Attack
        ));
    }

    #[test]
    fn duplicate_edges_are_deduped() {
        let m = build_model(&parse("[A]: a\n  + [B]: b\n\n[B]\n  +> [A]").unwrap());
        // `+ [B]` (B->A) and `+> [A]` under [B] (B->A) are the same edge.
        assert_eq!(m.edges.len(), 1);
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
    }

    #[test]
    fn pcs_interspersed_relation_produces_an_edge() {
        let m = build_model(&parse("<A>: d\n\n(1) [P]: p\n  +> [S]: s\n----\n(2) C").unwrap());
        // The interspersed relation: P supports S.
        assert!(has_edge(
            &m,
            stmt_node(&m, "P"),
            stmt_node(&m, "S"),
            RelationKind::Support
        ));
        // B4b roles are unaffected: P is a premise, C the main conclusion.
        let items = &m.pcs[0].items;
        assert_eq!(role_of(&items[0]), Role::Premise);
        assert_eq!(role_of(items.last().unwrap()), Role::MainConclusion);
    }

    #[test]
    fn unindented_relation_attaches_to_the_statement_above() {
        // Our parser accepts an unindented (`indent 0`) relation; it must still
        // attach to the statement above it (content anchors sit below indent 0).
        let m = build_model(&parse("[A]: a\n+ [B]: b").unwrap());
        assert_eq!(m.edges.len(), 1);
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
        assert!(m.relation_issues.is_empty());
    }

    #[test]
    fn sibling_relations_share_their_parent() {
        let m = build_model(&parse("[A]: a\n  + [B]: b\n  - [C]: c").unwrap());
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
        assert!(has_edge(
            &m,
            stmt_node(&m, "C"),
            stmt_node(&m, "A"),
            RelationKind::Attack
        ));
    }

    #[test]
    fn sibling_after_nested_dedents_to_ancestor() {
        // D dedents back to A's level after C nested under B.
        let m = build_model(&parse("[A]: a\n  + [B]: b\n    - [C]: c\n  - [D]: d").unwrap());
        assert!(has_edge(
            &m,
            stmt_node(&m, "C"),
            stmt_node(&m, "B"),
            RelationKind::Attack
        ));
        assert!(has_edge(
            &m,
            stmt_node(&m, "D"),
            stmt_node(&m, "A"),
            RelationKind::Attack
        ));
    }

    #[test]
    fn undercut_relation_is_recorded() {
        let m = build_model(&parse("[A]: a\n  _ [B]: b").unwrap());
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Undercut
        ));
    }

    #[test]
    fn argument_reference_anchors_a_relation() {
        let m = build_model(&parse("<A>\n  + [B]: b").unwrap());
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            arg_node(&m, "A"),
            RelationKind::Support
        ));
    }

    #[test]
    fn a_heading_resets_the_relation_context() {
        // A heading breaks adjacency: a relation after it does not attach to a
        // statement before it.
        let m = build_model(&parse("[A]: a\n\n# H\n\n  + [B]: b").unwrap());
        assert!(m.edges.is_empty());
        assert!(
            m.relation_issues
                .iter()
                .any(|i| matches!(i, RelationIssue::RelationWithoutParent { .. }))
        );
    }

    #[test]
    fn relation_without_a_parent_is_an_issue() {
        // An indented relation with nothing above it has no parent.
        let m = build_model(&parse("  + [B]: b").unwrap());
        assert!(m.edges.is_empty());
        assert!(
            m.relation_issues
                .iter()
                .any(|i| matches!(i, RelationIssue::RelationWithoutParent { .. }))
        );
    }

    #[test]
    fn titled_relation_target_merges_with_top_level() {
        let m = build_model(&parse("[A]: a\n\n[B]: b\n\n[A]\n  + [B]").unwrap());
        // [B] as a relation target is the same node as the top-level [B].
        let b_classes = m
            .statements
            .iter()
            .filter(|s| s.title.as_deref() == Some("B"))
            .count();
        assert_eq!(b_classes, 1);
        assert!(has_edge(
            &m,
            stmt_node(&m, "B"),
            stmt_node(&m, "A"),
            RelationKind::Support
        ));
    }

    #[test]
    fn model_prefixes_correspond_to_b3_and_b4a() {
        let doc = parse("[S]: s\n\n<A>: a\n\n(1) [P]: p\n----\n(2) c").unwrap();
        let m = build_model(&doc);
        let b3 = crate::build_statements(&doc);
        let b4a = crate::build_arguments(&doc);
        // The Model's statement/argument prefixes reuse B3/B4a ids and content.
        for s in &b3.statements {
            assert_eq!(
                m.statements[s.id.0].title.as_deref(),
                Some(s.title.as_str())
            );
            assert_eq!(m.statements[s.id.0].canonical_text, s.canonical_text);
        }
        for a in &b4a.arguments {
            assert_eq!(m.arguments[a.id.0].title.as_deref(), Some(a.title.as_str()));
        }
    }
}
