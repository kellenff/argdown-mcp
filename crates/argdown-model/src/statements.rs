//! Statement equivalence-class model (Layer B, slice B3).
//!
//! Turns the flat `Block::Statement` AST into a registry of unique
//! statement entities (one per title that appears in the document) plus a
//! block→entity assignment. Pure and total — strictness ("first definition
//! wins; later definitions are conflicts") is surfaced as data on
//! [`Statements::conflicts`], not as a `Result` failure. Inline statement
//! mentions (`StatementMention` in inlines) are not entities; B3 handles
//! block-level statements only.

use argdown_core::{Block, Document, Span};

pub use crate::metadata::Value;

/// Stable, source-order id; indexes `Statements::statements`.
///
/// Stable within a single parse only (the source is re-parsed fresh each
/// time); not designed to survive edits. Matches `SectionId` from B1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatementId(pub usize);

/// One statement entity in the equivalence class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub id: StatementId,
    /// Always `Some` — only titled statements form entities. Plain-text
    /// (untitled) statements are not in the model.
    pub title: String,
    /// First definition's text, or `None` if the entity is referenced
    /// but never defined in this document.
    pub canonical_text: Option<String>,
    /// First definition's metadata, parsed via B2's `parse_metadata`;
    /// `None` if no definition, the definition had no metadata block, or
    /// `parse_metadata` returned an error (B3 is total — B2 errors are
    /// absorbed as "no parsed metadata" rather than propagated).
    pub canonical_metadata: Option<Value>,
}

/// A redefinition conflict: a title was defined more than once. Surfaced
/// as data on `Statements::conflicts`, not as a `Result` failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementConflict {
    pub title: String,
    /// Source span of the first (canonical) definition.
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

/// Build the statement equivalence-class model for a parsed document.
///
/// Single pass over `document.blocks`, maintaining a `title → StatementId`
/// map. An entity is created on first occurrence of a title (definition
/// or reference — whichever comes first); the canonical is filled in on
/// the first definition; later definitions append to a per-title
/// `StatementConflict`. Plain-text statements and non-statement blocks
/// push `None` to `block_statements`.
pub fn build_statements(document: &Document) -> Statements {
    use std::collections::HashMap;

    let mut statements: Vec<Statement> = Vec::new();
    let mut by_title: HashMap<String, StatementId> = HashMap::new();
    // Records the span of the first definition for each titled statement.
    // Populated on the first definition; looked up when a redefinition
    // creates a conflict entry.
    let mut canonical_spans: HashMap<String, Span> = HashMap::new();
    // Conflicts keyed by title; only populated when a redefinition is
    // detected. Drained and sorted at the end so the output is in source
    // order of the title's first appearance.
    let mut conflict_map: HashMap<String, StatementConflict> = HashMap::new();
    let mut block_statements: Vec<Option<StatementId>> = Vec::with_capacity(document.blocks.len());

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
        if let (Block::Statement(s), Some(id)) = (block, id)
            && !s.is_reference
        {
            let title = s
                .title
                .as_ref()
                .expect("a statement block with a resolved id has a title");
            let entry = &mut statements[id.0];
            if entry.canonical_text.is_none() {
                // First definition: fill in canonical and record the
                // span for any future redefinition conflict.
                entry.canonical_text = Some(s.text.clone());
                entry.canonical_metadata = s
                    .metadata
                    .as_ref()
                    .map(crate::metadata::parse_metadata)
                    .transpose()
                    .ok()
                    .flatten();
                canonical_spans.insert(title.clone(), s.span);
            } else {
                // Already defined: this is a redefinition conflict.
                // Look up the canonical span recorded on the first
                // definition (it must exist — canonical_text was set
                // then).
                let canonical_span = canonical_spans
                    .get(title)
                    .copied()
                    .expect("a redefined title has a recorded canonical span");
                let entry =
                    conflict_map
                        .entry(title.clone())
                        .or_insert_with(|| StatementConflict {
                            title: title.clone(),
                            canonical_span,
                            conflicting_spans: Vec::new(),
                        });
                entry.conflicting_spans.push(s.span);
            }
        }

        // 3. Record the block→entity mapping.
        block_statements.push(id);
    }

    // Drain conflicts in source order (by first appearance of the title).
    let mut conflicts: Vec<StatementConflict> = conflict_map.into_values().collect();
    conflicts.sort_by_key(|c| {
        statements
            .iter()
            .position(|s| s.title == c.title)
            .unwrap_or(0)
    });

    Statements {
        statements,
        block_statements,
        conflicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_parser::parse;

    #[test]
    fn single_titled_definition_creates_one_entity() {
        let doc = parse("[A]: claim").unwrap();
        let s = build_statements(&doc);

        assert_eq!(s.statements.len(), 1);
        assert_eq!(s.statements[0].id, StatementId(0));
        assert_eq!(s.statements[0].title, "A");
        assert_eq!(s.statements[0].canonical_text.as_deref(), Some("claim"));
        assert_eq!(s.statements[0].canonical_metadata, None);
        assert!(s.conflicts.is_empty());
        assert_eq!(s.block_statements, vec![Some(StatementId(0))]);
    }
}
