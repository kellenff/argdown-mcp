//! Statement equivalence-class model (Layer B, slice B3).
//!
//! Turns the flat `Block::Statement` AST into a registry of unique
//! statement entities (one per title that appears in the document) plus a
//! block→entity assignment. Pure and total — strictness ("first definition
//! wins; later definitions are conflicts") is surfaced as data on
//! [`Statements::conflicts`], not as a `Result` failure. Inline statement
//! mentions (`StatementMention` in inlines) are not entities; B3 handles
//! block-level statements only.

use argdown_core::{Document, Span};

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
pub fn build_statements(_document: &Document) -> Statements {
    Statements::default()
}
