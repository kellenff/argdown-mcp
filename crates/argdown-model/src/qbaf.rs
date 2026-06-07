//! Quantitative Bipolar Argumentation Framework (QBAF) projection (Layer B, v1.1).
//!
//! Projects the [`Model`] into a weighted bipolar framework: nodes are arguments
//! with base degrees; edges are argument→argument `Support`, `Attack`, and
//! `Undercut` relations (`Undercut` maps to attack). Statement-level endpoints
//! are excluded — the same argument-only scope as the sharp Dung projection.
//!
//! Weight resolution (option C):
//! - `base_degree(arg)` ← `metadata.weight` on the argument, else [`DEFAULT_BASE`].
//! - `edge_weight(edge)` ← `metadata.weight` on the relation, else the source
//!   argument's base degree.

use crate::{ArgumentId, Model, Node, RelationKind, Value};

/// Default base degree when an argument carries no `weight` metadata.
pub const DEFAULT_BASE: f64 = 0.5;

/// Kind of a QBAF edge: attack (including undercut) or support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum QbafEdgeKind {
    Attack,
    Support,
}

/// An argument node with its base degree (prior to DF-QuAD propagation).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QbafNode {
    pub id: ArgumentId,
    pub base_degree: f64,
}

/// A weighted directed edge between two arguments.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QbafEdge {
    pub from: ArgumentId,
    pub to: ArgumentId,
    pub kind: QbafEdgeKind,
    pub weight: f64,
}

/// A weighted bipolar argumentation framework derived from the [`Model`].
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QbafFramework {
    pub nodes: Vec<QbafNode>,
    pub edges: Vec<QbafEdge>,
}

/// Parse a numeric `weight` from a metadata value.
pub fn parse_weight(value: &Value) -> Result<f64, String> {
    if let Some(n) = value.as_f64() {
        return Ok(n);
    }
    if let Value::String(s) = value {
        return s
            .parse::<f64>()
            .map_err(|_| format!("weight '{s}' is not a valid number"));
    }
    Err(format!("weight must be a number, got {value:?}"))
}

/// Resolve an argument's base degree from its `canonical_metadata`.
pub fn base_degree(meta: &Option<Value>) -> Result<f64, String> {
    match meta.as_ref().and_then(|v| v.get("weight")) {
        Some(w) => parse_weight(w),
        None => Ok(DEFAULT_BASE),
    }
}

/// Resolve an edge's weight: relation `weight` overrides the source base degree.
pub fn edge_weight(source_base: f64, edge_meta: &Option<Value>) -> Result<f64, String> {
    match edge_meta.as_ref().and_then(|v| v.get("weight")) {
        Some(w) => parse_weight(w),
        None => Ok(source_base),
    }
}

/// Project the Model into a QBAF framework — arguments as nodes, argument-level
/// `Support` / `Attack` / `Undercut` as weighted edges (`Undercut` → attack).
pub fn project_qbaf(model: &Model) -> Result<QbafFramework, String> {
    let mut base_by_arg: Vec<f64> = Vec::with_capacity(model.arguments.len());
    let mut nodes: Vec<QbafNode> = Vec::with_capacity(model.arguments.len());

    for arg in &model.arguments {
        let degree = base_degree(&arg.canonical_metadata)?;
        base_by_arg.push(degree);
        nodes.push(QbafNode {
            id: arg.id,
            base_degree: degree,
        });
    }

    let mut edges: Vec<QbafEdge> = Vec::new();
    for edge in &model.edges {
        let (Node::Argument(from), Node::Argument(to)) = (edge.from, edge.to) else {
            continue;
        };
        let kind = match edge.kind {
            RelationKind::Attack | RelationKind::Undercut => QbafEdgeKind::Attack,
            RelationKind::Support => QbafEdgeKind::Support,
            RelationKind::Contradictory => continue,
        };
        let source_base = base_by_arg
            .get(from.0)
            .copied()
            .ok_or_else(|| format!("edge source argument {:?} is out of range", from))?;
        let weight = edge_weight(source_base, &edge.canonical_metadata)?;
        let qbaf_edge = QbafEdge {
            from,
            to,
            kind,
            weight,
        };
        if !edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.kind == kind)
        {
            edges.push(qbaf_edge);
        }
    }

    Ok(QbafFramework { nodes, edges })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_model;
    use argdown_parser::parse;

    fn arg_id(m: &Model, title: &str) -> ArgumentId {
        m.arguments
            .iter()
            .find(|a| a.title.as_deref() == Some(title))
            .expect("argument")
            .id
    }

    #[test]
    fn no_metadata_uses_default_base() {
        let m = build_model(&parse("<a>: A\n\n<b>: B").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        assert_eq!(qbaf.nodes.len(), 2);
        assert!(qbaf.nodes.iter().all(|n| n.base_degree == DEFAULT_BASE));
    }

    #[test]
    fn argument_weight_sets_base_degree() {
        let m = build_model(&parse("<a>: A {weight: 0.8}\n\n<b>: B").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        let a = qbaf
            .nodes
            .iter()
            .find(|n| n.id == arg_id(&m, "a"))
            .expect("node a");
        assert!((a.base_degree - 0.8).abs() < f64::EPSILON);
        let b = qbaf
            .nodes
            .iter()
            .find(|n| n.id == arg_id(&m, "b"))
            .expect("node b");
        assert!((b.base_degree - DEFAULT_BASE).abs() < f64::EPSILON);
    }

    #[test]
    fn edge_weight_overrides_source_base() {
        let m = build_model(
            &parse("<a>: A {weight: 0.8}\n  + {weight: 0.3} <b>: B\n\n<b>: B").unwrap(),
        );
        let qbaf = project_qbaf(&m).unwrap();
        assert_eq!(qbaf.edges.len(), 1);
        let edge = &qbaf.edges[0];
        assert_eq!(edge.kind, QbafEdgeKind::Support);
        assert!((edge.weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn undercut_becomes_attack_edge() {
        let m = build_model(&parse("<a>: A\n  _ <b>\n\n<b>: B").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        assert_eq!(qbaf.edges.len(), 1);
        assert_eq!(qbaf.edges[0].kind, QbafEdgeKind::Attack);
        assert_eq!(qbaf.edges[0].from, arg_id(&m, "b"));
        assert_eq!(qbaf.edges[0].to, arg_id(&m, "a"));
    }

    #[test]
    fn argument_level_attack_is_included() {
        let m = build_model(&parse("<a>: A\n  - <b>\n\n<b>: B").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        assert_eq!(qbaf.edges.len(), 1);
        assert_eq!(qbaf.edges[0].kind, QbafEdgeKind::Attack);
    }

    #[test]
    fn statement_level_edges_are_excluded() {
        let m = build_model(&parse("<a>: A\n\n(1) p\n  - <b>\n----\n(2) c\n\n<b>: B").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        assert!(qbaf.edges.is_empty());
    }

    #[test]
    fn invalid_argument_weight_is_an_error() {
        let m = build_model(&parse("<a>: A {weight: bad}\n\n<b>: B").unwrap());
        let err = project_qbaf(&m).unwrap_err();
        assert!(err.contains("weight"));
    }

    #[test]
    fn edge_without_override_uses_source_base() {
        // Outbound `+>`: source is the parent argument (a), not the target (b).
        let m = build_model(&parse("<a>: A {weight: 0.8}\n  +> <b>: B\n\n<b>: B").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        assert_eq!(qbaf.edges.len(), 1);
        assert!((qbaf.edges[0].weight - 0.8).abs() < f64::EPSILON);
    }
}
