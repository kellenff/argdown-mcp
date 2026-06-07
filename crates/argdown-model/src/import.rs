//! Deserialize a Layer-B [`Model`] from JSON and YAML — the validating
//! counterpart to [`crate::export`].
//!
//! A derived `Deserialize` only checks that the input is *structurally* a
//! `Model`; it cannot know that the `Model`'s cross-references hold. So
//! [`from_json`]/[`from_yaml`] parse structurally and then run [`validate`]
//! before handing back a value: a successfully-returned `Model` is guaranteed
//! to satisfy the intra-`Model` invariants the rest of the crate relies on
//! (positional arena ids, in-range references, well-formed inference bindings).
//! This is the boundary where untrusted bytes become a trusted domain value —
//! parse, don't validate.
//!
//! What is checked is everything verifiable from the `Model` alone. Invariants
//! that are only meaningful against the originating `Document` (e.g.
//! `block_pcs` being index-aligned with `document.blocks`, or
//! prefix-correspondence with B3/B4a) cannot be re-derived here and are not
//! checked; to regenerate a `Model` with *those* guarantees, rebuild it from a
//! parsed `Document` via [`crate::build_model`].

use crate::{Model, Node, ResolvedPcsItem};
use std::fmt;

/// Deserialize a [`Model`] from JSON, validating its invariants.
pub fn from_json(s: &str) -> Result<Model, ImportError> {
    let model: Model = serde_json::from_str(s)?;
    validate(&model)?;
    Ok(model)
}

/// Deserialize a [`Model`] from YAML (via `noyalib`'s `serde_yaml`), validating
/// its invariants.
pub fn from_yaml(s: &str) -> Result<Model, ImportError> {
    let model: Model = noyalib::compat::serde_yaml::from_str(s)?;
    validate(&model)?;
    Ok(model)
}

/// A failure while importing a [`Model`]: either the bytes were not structurally
/// a `Model`, or the parsed `Model` broke an invariant.
#[derive(Debug)]
pub enum ImportError {
    Json(serde_json::Error),
    Yaml(noyalib::compat::serde_yaml::Error),
    Invariant(InvariantViolation),
}

/// One of the three arenas a reference can point into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arena {
    Statements,
    Arguments,
    Pcs,
}

/// Where a dangling reference was found — identifies the field and its location.
/// Each variant implies the arena (or item vector) it indexes into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefSource {
    /// `block_pcs[block]` → `pcs`.
    BlockPcs { block: usize },
    /// `arguments[argument].pcs` → `pcs`.
    ArgumentPcs { argument: usize },
    /// `pcs[pcs].argument` → `arguments`.
    PcsArgument { pcs: usize },
    /// `pcs[pcs].items[item].statement` → `statements`.
    PcsItemStatement { pcs: usize, item: usize },
    /// `pcs[pcs].items[item].concludes_item_idx` → that PCS's `items`.
    InferenceConclusion { pcs: usize, item: usize },
    /// `edges[edge].from` → `statements`/`arguments`.
    EdgeFrom { edge: usize },
    /// `edges[edge].to` → `statements`/`arguments`.
    EdgeTo { edge: usize },
}

/// A broken intra-`Model` invariant. The first one found stops validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    /// An arena entry's stable id does not equal its position (arena ids are
    /// positional and index the arena).
    MisplacedId {
        arena: Arena,
        index: usize,
        found: usize,
    },
    /// A reference points past the end of the collection it indexes.
    DanglingReference {
        source: RefSource,
        index: usize,
        len: usize,
    },
    /// An inference's bound conclusion index does not point at a statement item.
    ConclusionNotAStatement { pcs: usize, item: usize },
}

/// Check every intra-`Model` cross-reference; return the first violation, if any.
fn validate(model: &Model) -> Result<(), InvariantViolation> {
    use InvariantViolation::*;

    let n_statements = model.statements.len();
    let n_arguments = model.arguments.len();
    let n_pcs = model.pcs.len();

    // Arena ids must equal their position.
    for (i, s) in model.statements.iter().enumerate() {
        if s.id.0 != i {
            return Err(MisplacedId {
                arena: Arena::Statements,
                index: i,
                found: s.id.0,
            });
        }
    }
    for (i, a) in model.arguments.iter().enumerate() {
        if a.id.0 != i {
            return Err(MisplacedId {
                arena: Arena::Arguments,
                index: i,
                found: a.id.0,
            });
        }
        if let Some(pcs) = a.pcs
            && pcs.0 >= n_pcs
        {
            return Err(DanglingReference {
                source: RefSource::ArgumentPcs { argument: i },
                index: pcs.0,
                len: n_pcs,
            });
        }
    }
    for (i, p) in model.pcs.iter().enumerate() {
        if p.id.0 != i {
            return Err(MisplacedId {
                arena: Arena::Pcs,
                index: i,
                found: p.id.0,
            });
        }
    }

    // PCS owners, statement references, and inference bindings.
    for (pi, p) in model.pcs.iter().enumerate() {
        if p.argument.0 >= n_arguments {
            return Err(DanglingReference {
                source: RefSource::PcsArgument { pcs: pi },
                index: p.argument.0,
                len: n_arguments,
            });
        }
        let items_len = p.items.len();
        for (ii, item) in p.items.iter().enumerate() {
            match item {
                ResolvedPcsItem::Statement { statement, .. } => {
                    if statement.0 >= n_statements {
                        return Err(DanglingReference {
                            source: RefSource::PcsItemStatement { pcs: pi, item: ii },
                            index: statement.0,
                            len: n_statements,
                        });
                    }
                }
                ResolvedPcsItem::Inference {
                    concludes_item_idx, ..
                } => {
                    if let Some(c) = *concludes_item_idx {
                        if c >= items_len {
                            return Err(DanglingReference {
                                source: RefSource::InferenceConclusion { pcs: pi, item: ii },
                                index: c,
                                len: items_len,
                            });
                        }
                        if !matches!(p.items[c], ResolvedPcsItem::Statement { .. }) {
                            return Err(ConclusionNotAStatement { pcs: pi, item: ii });
                        }
                    }
                }
                ResolvedPcsItem::Relation(_) => {}
            }
        }
    }

    // block_pcs back-references.
    for (bi, slot) in model.block_pcs.iter().enumerate() {
        if let Some(pcs) = slot
            && pcs.0 >= n_pcs
        {
            return Err(DanglingReference {
                source: RefSource::BlockPcs { block: bi },
                index: pcs.0,
                len: n_pcs,
            });
        }
    }

    // Edge endpoints.
    for (ei, e) in model.edges.iter().enumerate() {
        check_node(
            e.from,
            RefSource::EdgeFrom { edge: ei },
            n_statements,
            n_arguments,
        )?;
        check_node(
            e.to,
            RefSource::EdgeTo { edge: ei },
            n_statements,
            n_arguments,
        )?;
    }

    Ok(())
}

/// A node endpoint resolves into the statements or arguments arena by variant.
fn check_node(
    node: Node,
    source: RefSource,
    n_statements: usize,
    n_arguments: usize,
) -> Result<(), InvariantViolation> {
    let (index, len) = match node {
        Node::Statement(id) => (id.0, n_statements),
        Node::Argument(id) => (id.0, n_arguments),
    };
    if index >= len {
        return Err(InvariantViolation::DanglingReference { source, index, len });
    }
    Ok(())
}

impl fmt::Display for Arena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Arena::Statements => "statements",
            Arena::Arguments => "arguments",
            Arena::Pcs => "pcs",
        })
    }
}

impl fmt::Display for RefSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefSource::BlockPcs { block } => write!(f, "block_pcs[{block}]"),
            RefSource::ArgumentPcs { argument } => write!(f, "arguments[{argument}].pcs"),
            RefSource::PcsArgument { pcs } => write!(f, "pcs[{pcs}].argument"),
            RefSource::PcsItemStatement { pcs, item } => {
                write!(f, "pcs[{pcs}].items[{item}].statement")
            }
            RefSource::InferenceConclusion { pcs, item } => {
                write!(f, "pcs[{pcs}].items[{item}].concludes_item_idx")
            }
            RefSource::EdgeFrom { edge } => write!(f, "edges[{edge}].from"),
            RefSource::EdgeTo { edge } => write!(f, "edges[{edge}].to"),
        }
    }
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvariantViolation::MisplacedId {
                arena,
                index,
                found,
            } => write!(
                f,
                "{arena}[{index}] has id {found}, but arena ids must equal their position"
            ),
            InvariantViolation::DanglingReference { source, index, len } => {
                write!(f, "{source} references index {index}, but only {len} exist")
            }
            InvariantViolation::ConclusionNotAStatement { pcs, item } => write!(
                f,
                "pcs[{pcs}].items[{item}] is an inference whose bound conclusion is not a statement"
            ),
        }
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::Json(e) => write!(f, "JSON deserialization failed: {e}"),
            ImportError::Yaml(e) => write!(f, "YAML deserialization failed: {e}"),
            ImportError::Invariant(v) => write!(f, "model invariant violated: {v}"),
        }
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImportError::Json(e) => Some(e),
            ImportError::Yaml(e) => Some(e),
            ImportError::Invariant(_) => None,
        }
    }
}

impl From<serde_json::Error> for ImportError {
    fn from(e: serde_json::Error) -> Self {
        ImportError::Json(e)
    }
}

impl From<noyalib::compat::serde_yaml::Error> for ImportError {
    fn from(e: noyalib::compat::serde_yaml::Error) -> Self {
        ImportError::Yaml(e)
    }
}

impl From<InvariantViolation> for ImportError {
    fn from(v: InvariantViolation) -> Self {
        ImportError::Invariant(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_model, to_json, to_yaml};
    use argdown_parser::parse;
    use serde_json::{Value as Json, json};

    /// Statements, a named argument with a one-step PCS, and a support edge.
    const SAMPLE: &str = "<A>: d\n\n(1) P1\n----\n(2) C1\n\n[X]: x\n  + [Y]: y";

    /// Reparse the sample's JSON into a mutable `serde_json::Value`.
    fn sample_json() -> Json {
        let model = build_model(&parse(SAMPLE).unwrap());
        serde_json::from_str(&to_json(&model).unwrap()).unwrap()
    }

    #[test]
    fn json_round_trips_to_an_equal_model() {
        let model = build_model(&parse(SAMPLE).unwrap());
        let back = from_json(&to_json(&model).unwrap()).expect("valid round-trip");
        assert_eq!(back, model);
    }

    #[test]
    fn yaml_round_trips_to_an_equal_model() {
        let model = build_model(&parse(SAMPLE).unwrap());
        let back = from_yaml(&to_yaml(&model).unwrap()).expect("valid round-trip");
        assert_eq!(back, model);
    }

    #[test]
    fn structurally_invalid_json_is_a_deserialize_error() {
        let err = from_json("{ not valid").unwrap_err();
        assert!(matches!(err, ImportError::Json(_)));
    }

    #[test]
    fn out_of_range_edge_endpoint_is_rejected() {
        let mut v = sample_json();
        v["edges"][0]["from"]["Statement"] = json!(999);
        let err = from_json(&v.to_string()).unwrap_err();
        assert!(matches!(
            err,
            ImportError::Invariant(InvariantViolation::DanglingReference {
                source: RefSource::EdgeFrom { edge: 0 },
                index: 999,
                ..
            })
        ));
    }

    #[test]
    fn misplaced_arena_id_is_rejected() {
        let mut v = sample_json();
        v["statements"][0]["id"] = json!(7);
        let err = from_json(&v.to_string()).unwrap_err();
        assert!(matches!(
            err,
            ImportError::Invariant(InvariantViolation::MisplacedId {
                arena: Arena::Statements,
                index: 0,
                found: 7,
            })
        ));
    }

    #[test]
    fn inference_conclusion_pointing_at_a_non_statement_is_rejected() {
        // items: 0=Statement 1=Inference 2=Statement. Point the inference's
        // conclusion at itself (index 1) — in range, but not a statement.
        let mut v = sample_json();
        v["pcs"][0]["items"][1]["Inference"]["concludes_item_idx"] = json!(1);
        let err = from_json(&v.to_string()).unwrap_err();
        assert!(matches!(
            err,
            ImportError::Invariant(InvariantViolation::ConclusionNotAStatement { pcs: 0, item: 1 })
        ));
    }

    #[test]
    fn out_of_range_inference_conclusion_is_rejected() {
        let mut v = sample_json();
        v["pcs"][0]["items"][1]["Inference"]["concludes_item_idx"] = json!(99);
        let err = from_json(&v.to_string()).unwrap_err();
        assert!(matches!(
            err,
            ImportError::Invariant(InvariantViolation::DanglingReference {
                source: RefSource::InferenceConclusion { pcs: 0, item: 1 },
                index: 99,
                ..
            })
        ));
    }
}
