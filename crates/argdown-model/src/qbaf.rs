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

use std::collections::HashMap;

use crate::{ArgumentId, Model, Node, RelationKind, Value};

/// Default iteration cap for DF-QuAD fixpoint search.
pub const DEFAULT_MAX_ITERATIONS: usize = 500;

const DEGREE_EPSILON: f64 = 1e-9;

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

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Classify a converged final degree against `threshold`.
/// When `oscillated` is true (fixpoint not reached), returns `"undec"`.
pub fn classify_degree(final_degree: f64, threshold: f64, oscillated: bool) -> &'static str {
    if oscillated {
        "undec"
    } else if final_degree >= threshold {
        "accepted"
    } else {
        "rejected"
    }
}

/// Outcome of a DF-QuAD run, including whether a fixpoint was reached.
#[derive(Debug, Clone, PartialEq)]
pub struct DfQuadResult {
    pub degrees: HashMap<ArgumentId, f64>,
    pub converged: bool,
}

fn df_quad_step(
    qbaf: &QbafFramework,
    degrees: &HashMap<ArgumentId, f64>,
) -> Result<HashMap<ArgumentId, f64>, String> {
    let mut next = degrees.clone();
    for node in &qbaf.nodes {
        let mut attack_sum = 0.0;
        let mut support_sum = 0.0;
        for edge in &qbaf.edges {
            if edge.to != node.id {
                continue;
            }
            let parent = degrees
                .get(&edge.from)
                .copied()
                .ok_or_else(|| format!("edge source argument {:?} is missing", edge.from))?;
            let contribution = parent * edge.weight;
            match edge.kind {
                QbafEdgeKind::Attack => attack_sum += contribution,
                QbafEdgeKind::Support => support_sum += contribution,
            }
        }
        let updated = clamp01(node.base_degree + support_sum - attack_sum);
        next.insert(node.id, updated);
    }
    Ok(next)
}

fn degrees_converged(
    qbaf: &QbafFramework,
    old: &HashMap<ArgumentId, f64>,
    new: &HashMap<ArgumentId, f64>,
) -> bool {
    qbaf.nodes.iter().all(|n| {
        let prev = old.get(&n.id).copied().unwrap_or(0.0);
        let curr = new.get(&n.id).copied().unwrap_or(0.0);
        (prev - curr).abs() < DEGREE_EPSILON
    })
}

/// DF-QuAD iterative update: supports increase, attacks decrease, clamp to \[0, 1\].
pub fn df_quad_run(qbaf: &QbafFramework, max_iterations: usize) -> Result<DfQuadResult, String> {
    if qbaf.nodes.is_empty() {
        return Ok(DfQuadResult {
            degrees: HashMap::new(),
            converged: true,
        });
    }

    let mut degrees: HashMap<ArgumentId, f64> = qbaf
        .nodes
        .iter()
        .map(|n| (n.id, n.base_degree))
        .collect();

    for _ in 0..max_iterations {
        let next = df_quad_step(qbaf, &degrees)?;
        if degrees_converged(qbaf, &degrees, &next) {
            return Ok(DfQuadResult {
                degrees: next,
                converged: true,
            });
        }
        degrees = next;
    }

    Ok(DfQuadResult {
        degrees,
        converged: false,
    })
}

/// DF-QuAD iterative update: supports increase, attacks decrease, clamp to \[0, 1\].
/// Returns `Err` when the fixpoint is not reached within `max_iterations`.
pub fn df_quad(
    qbaf: &QbafFramework,
    max_iterations: usize,
) -> Result<HashMap<ArgumentId, f64>, String> {
    let result = df_quad_run(qbaf, max_iterations)?;
    if result.converged {
        Ok(result.degrees)
    } else {
        Err(format!(
            "DF-QuAD did not converge within {max_iterations} iterations"
        ))
    }
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

    #[test]
    fn df_quad_support_increases_degree() {
        // B (0.4) supports A (0.6) with edge weight 0.2 → A = clamp(0.6 + 0.4*0.2) = 0.68.
        let m = build_model(
            &parse("<a>: A {weight: 0.6}\n  + {weight: 0.2} <b>\n\n<b>: B {weight: 0.4}").unwrap(),
        );
        let qbaf = project_qbaf(&m).unwrap();
        let degrees = df_quad(&qbaf, DEFAULT_MAX_ITERATIONS).unwrap();
        let a = arg_id(&m, "a");
        assert!((degrees[&a] - 0.68).abs() < 1e-6);
    }

    #[test]
    fn df_quad_attack_decreases_degree() {
        // B (0.5) attacks A (0.8) with edge weight 0.5 → A = clamp(0.8 - 0.5*0.5) = 0.55.
        let m = build_model(&parse("<a>: A {weight: 0.8}\n  - <b>\n\n<b>: B {weight: 0.5}").unwrap());
        let qbaf = project_qbaf(&m).unwrap();
        let degrees = df_quad(&qbaf, DEFAULT_MAX_ITERATIONS).unwrap();
        let a = arg_id(&m, "a");
        assert!((degrees[&a] - 0.55).abs() < 1e-6);
    }

    #[test]
    fn df_quad_clamps_to_unit_interval() {
        let m = build_model(
            &parse("<a>: A {weight: 0.9}\n  + {weight: 1.0} <b>\n\n<b>: B {weight: 1.0}").unwrap(),
        );
        let qbaf = project_qbaf(&m).unwrap();
        let degrees = df_quad(&qbaf, DEFAULT_MAX_ITERATIONS).unwrap();
        let a = arg_id(&m, "a");
        assert!((degrees[&a] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn classify_degree_respects_threshold_and_oscillation() {
        assert_eq!(classify_degree(0.6, 0.5, false), "accepted");
        assert_eq!(classify_degree(0.49, 0.5, false), "rejected");
        assert_eq!(classify_degree(0.9, 0.5, true), "undec");
    }

    #[test]
    fn df_quad_non_convergence_is_an_error() {
        // Three-argument support chain needs >1 pass; cap at 1 iteration.
        let m = build_model(
            &parse(
                "<a>: A {weight: 0.5}\n  + <b>\n\n<b>: B {weight: 0.5}\n  + <c>\n\n<c>: C {weight: 0.5}",
            )
            .unwrap(),
        );
        let qbaf = project_qbaf(&m).unwrap();
        let err = df_quad(&qbaf, 1).unwrap_err();
        assert!(err.contains("did not converge"));
    }
}
