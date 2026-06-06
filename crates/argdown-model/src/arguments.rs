//! Argument equivalence-class model (Layer B, slice B4a).
//!
//! Turns the flat `Block::Argument` AST into a registry of unique argument
//! entities (one per `<Title>` that appears as a top-level block) plus a
//! block→entity assignment. Pure and total — strictness ("first definition
//! wins; later definitions are conflicts") is surfaced as data on
//! [`Arguments::conflicts`], not as a `Result` failure. Arguments appearing
//! only as relation targets or inline mentions (`ArgumentMention`) are not
//! entities; B4a handles top-level argument blocks only.

use argdown_core::{Block, Document, Span};

pub use crate::metadata::Value;

/// Stable, source-order id; indexes `Arguments::arguments`.
///
/// Stable within a single parse only (the source is re-parsed fresh each
/// time); not designed to survive edits. Matches `StatementId` from B3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArgumentId(pub usize);

/// One argument entity in the equivalence class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    pub id: ArgumentId,
    /// Arguments are always titled (`argdown_core::Argument.title: String`),
    /// so unlike B3 statements there is no untitled case.
    pub title: String,
    /// First definition's description, or `None` if the entity is referenced
    /// but never defined in this document. `Some("")` (defined as empty) is
    /// distinct from `None` (referenced only).
    pub canonical_description: Option<String>,
    /// First definition's metadata, parsed via B2's `parse_metadata`;
    /// `None` if no definition, the definition had no metadata block, or
    /// `parse_metadata` returned an error (B4a is total — B2 errors are
    /// absorbed as "no parsed metadata" rather than propagated).
    pub canonical_metadata: Option<Value>,
}

/// A redefinition conflict: a title was defined more than once. Surfaced
/// as data on `Arguments::conflicts`, not as a `Result` failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentConflict {
    pub title: String,
    /// Source span of the first (canonical) definition.
    pub canonical_span: Span,
    /// Source spans of every later (conflicting) definition, in source
    /// order.
    pub conflicting_spans: Vec<Span>,
}

/// The B4a output: a flat argument arena, a block→argument assignment,
/// and a list of redefinition conflicts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Arguments {
    /// Flat arena. `ArgumentId(i)` indexes `arguments[i]`. Source order
    /// of first occurrence (definition or reference — whichever comes
    /// first in the document).
    pub arguments: Vec<Argument>,
    /// Index-aligned with `document.blocks`: the argument entity for each
    /// top-level `Block::Argument`, or `None` for every other block kind
    /// (Heading, Statement, Relation, Pcs).
    pub block_arguments: Vec<Option<ArgumentId>>,
    /// Redefinition conflicts found while walking the document, in source
    /// order (sorted by the order their title first appeared).
    pub conflicts: Vec<ArgumentConflict>,
}

/// Build the argument equivalence-class model for a parsed document.
///
/// Single pass over `document.blocks`, maintaining a `title → ArgumentId`
/// map. An entity is created on first occurrence of a title (definition
/// or reference — whichever comes first); the canonical is filled in on
/// the first definition; later definitions append to a per-title
/// `ArgumentConflict`. Non-argument blocks push `None` to `block_arguments`.
pub fn build_arguments(document: &Document) -> Arguments {
    use std::collections::HashMap;

    let mut arguments: Vec<Argument> = Vec::new();
    let mut by_title: HashMap<String, ArgumentId> = HashMap::new();
    // Records the span of the first definition for each titled argument.
    // Populated on the first definition; looked up when a redefinition
    // creates a conflict entry.
    let mut canonical_spans: HashMap<String, Span> = HashMap::new();
    // Conflicts keyed by title; only populated when a redefinition is
    // detected. Drained and sorted at the end so the output is in source
    // order of the title's first appearance.
    let mut conflict_map: HashMap<String, ArgumentConflict> = HashMap::new();
    let mut block_arguments: Vec<Option<ArgumentId>> = Vec::with_capacity(document.blocks.len());

    for block in &document.blocks {
        // 1. Resolve the block's id (if any). Arguments are always titled,
        //    so every `Block::Argument` resolves to an entity.
        let id = match block {
            Block::Argument(a) => {
                let id = *by_title.entry(a.title.clone()).or_insert_with(|| {
                    let id = ArgumentId(arguments.len());
                    arguments.push(Argument {
                        id,
                        title: a.title.clone(),
                        canonical_description: None,
                        canonical_metadata: None,
                    });
                    id
                });
                Some(id)
            }
            _ => None,
        };

        // 2. For a definition, fill in canonical on first occurrence and
        //    record a conflict on a redefinition.
        if let (Block::Argument(a), Some(id)) = (block, id)
            && !a.is_reference
        {
            let entry = &mut arguments[id.0];
            if entry.canonical_description.is_none() {
                // First definition: fill in canonical and record the
                // span for any future redefinition conflict.
                entry.canonical_description = Some(a.description.clone());
                entry.canonical_metadata = a
                    .metadata
                    .as_ref()
                    .map(crate::metadata::parse_metadata)
                    .transpose()
                    .ok()
                    .flatten();
                canonical_spans.insert(a.title.clone(), a.span);
            } else {
                // Already defined: this is a redefinition conflict.
                // Look up the canonical span recorded on the first
                // definition (it must exist — canonical_description was
                // set then).
                let canonical_span = canonical_spans
                    .get(&a.title)
                    .copied()
                    .expect("a redefined title has a recorded canonical span");
                let entry =
                    conflict_map
                        .entry(a.title.clone())
                        .or_insert_with(|| ArgumentConflict {
                            title: a.title.clone(),
                            canonical_span,
                            conflicting_spans: Vec::new(),
                        });
                entry.conflicting_spans.push(a.span);
            }
        }

        // 3. Record the block→entity mapping.
        block_arguments.push(id);
    }

    // Drain conflicts in source order (by first appearance of the title).
    let mut conflicts: Vec<ArgumentConflict> = conflict_map.into_values().collect();
    conflicts.sort_by_key(|c| {
        arguments
            .iter()
            .position(|a| a.title == c.title)
            .unwrap_or(0)
    });

    Arguments {
        arguments,
        block_arguments,
        conflicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_parser::parse;

    #[test]
    fn single_argument_definition_creates_one_entity() {
        let doc = parse("<A>: desc").unwrap();
        let a = build_arguments(&doc);

        assert_eq!(a.arguments.len(), 1);
        assert_eq!(a.arguments[0].id, ArgumentId(0));
        assert_eq!(a.arguments[0].title, "A");
        assert_eq!(
            a.arguments[0].canonical_description.as_deref(),
            Some("desc")
        );
        assert_eq!(a.arguments[0].canonical_metadata, None);
        assert!(a.conflicts.is_empty());
        assert_eq!(a.block_arguments, vec![Some(ArgumentId(0))]);
    }

    #[test]
    fn empty_document_has_no_arguments() {
        let doc = parse("").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a, Arguments::default());
    }

    #[test]
    fn single_argument_reference_has_no_canonical() {
        let doc = parse("<A>").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 1);
        assert_eq!(a.arguments[0].title, "A");
        assert_eq!(a.arguments[0].canonical_description, None);
        assert!(a.conflicts.is_empty());
        assert_eq!(a.block_arguments, vec![Some(ArgumentId(0))]);
    }

    #[test]
    fn definition_then_reference_share_one_entity() {
        let doc = parse("<A>: desc\n\n<A>").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 1);
        assert_eq!(
            a.arguments[0].canonical_description.as_deref(),
            Some("desc")
        );
        assert_eq!(
            a.block_arguments,
            vec![Some(ArgumentId(0)), Some(ArgumentId(0))]
        );
        assert!(a.conflicts.is_empty());
    }

    #[test]
    fn reference_then_definition_fills_canonical_later() {
        let doc = parse("<A>\n\n<A>: desc").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 1);
        // The reference created the entity; the later definition filled canonical.
        assert_eq!(
            a.arguments[0].canonical_description.as_deref(),
            Some("desc")
        );
        assert_eq!(
            a.block_arguments,
            vec![Some(ArgumentId(0)), Some(ArgumentId(0))]
        );
        assert!(a.conflicts.is_empty());
    }

    #[test]
    fn redefinition_records_a_conflict() {
        let doc = parse("<A>: d1\n\n<A>: d2").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 1);
        // First definition wins.
        assert_eq!(a.arguments[0].canonical_description.as_deref(), Some("d1"));
        assert_eq!(a.conflicts.len(), 1);
        assert_eq!(a.conflicts[0].title, "A");
        assert_eq!(a.conflicts[0].conflicting_spans.len(), 1);
    }

    #[test]
    fn three_distinct_titles_create_three_entities_in_source_order() {
        let doc = parse("<A>: one\n\n<B>: two\n\n<C>: three").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 3);
        assert_eq!(a.arguments[0].title, "A");
        assert_eq!(a.arguments[1].title, "B");
        assert_eq!(a.arguments[2].title, "C");
        assert_eq!(
            a.block_arguments,
            vec![
                Some(ArgumentId(0)),
                Some(ArgumentId(1)),
                Some(ArgumentId(2)),
            ]
        );
        assert!(a.conflicts.is_empty());
    }

    #[test]
    fn non_argument_blocks_have_no_argument_id() {
        // A heading, a titled statement, and a PCS — none is an argument.
        let doc =
            parse("# heading\n\n[S]: a statement\n\n(1) premise\n-----\n(2) conclusion").unwrap();
        let a = build_arguments(&doc);
        assert!(a.arguments.is_empty());
        assert_eq!(a.block_arguments.len(), doc.blocks.len());
        assert!(a.block_arguments.iter().all(Option::is_none));
        assert!(a.conflicts.is_empty());
    }

    #[test]
    fn three_redefinitions_record_two_conflicting_spans() {
        let doc = parse("<A>: d1\n\n<A>: d2\n\n<A>: d3").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 1);
        assert_eq!(a.arguments[0].canonical_description.as_deref(), Some("d1"));
        assert_eq!(a.conflicts.len(), 1);
        assert_eq!(a.conflicts[0].conflicting_spans.len(), 2);
    }

    #[test]
    fn canonical_metadata_is_parsed() {
        // Inline metadata on the definition: the parser captures it as
        // Argument.metadata; build_arguments parses it via B2's
        // parse_metadata and stores the result as canonical_metadata.
        let doc = parse("<A>: desc { key: value }").unwrap();
        let a = build_arguments(&doc);
        let meta = a.arguments[0]
            .canonical_metadata
            .as_ref()
            .expect("definition had metadata");
        let Value::Mapping(map) = meta else {
            panic!("expected Value::Mapping, got {meta:?}");
        };
        assert!(map.contains_key("key"));
    }

    #[test]
    fn parser_normalizes_titles_by_trimming() {
        // The parser's `argument_title` already trims whitespace; B4a
        // doesn't re-normalize, so the trim is inherited. This test
        // documents that expectation.
        let doc = parse("< A >: desc").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 1);
        assert_eq!(a.arguments[0].title, "A");
    }

    #[test]
    fn conflicts_are_sorted_in_source_order_of_title_first_appearance() {
        // Titles X, Y, Z appear in order X, Y, Z. Each is redefined in the
        // same order. Conflicts should come out in source order: X, Y, Z
        // (the order the titles first appeared, not the order the
        // redefinitions happened).
        let doc = parse("<X>: x1\n\n<Y>: y1\n\n<Z>: z1\n\n<X>: x2\n\n<Y>: y2\n\n<Z>: z2").unwrap();
        let a = build_arguments(&doc);
        assert_eq!(a.arguments.len(), 3);
        assert_eq!(a.conflicts.len(), 3);
        assert_eq!(a.conflicts[0].title, "X");
        assert_eq!(a.conflicts[1].title, "Y");
        assert_eq!(a.conflicts[2].title, "Z");
    }
}
