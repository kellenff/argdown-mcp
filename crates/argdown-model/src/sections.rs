//! Section assembly (Layer B, slice B1).
//!
//! Turns the flat parsed [`Document`] into a nested section tree plus a
//! block→section assignment. Pure and total: computed from `&Document`, the AST
//! is never mutated. Sections are a flat arena navigated by [`SectionId`].

use argdown_core::{Document, Span};

/// Stable, source-order id; indexes [`Sections::sections`].
///
/// Stable within a single parse only (the source is re-parsed fresh each time);
/// not designed to survive edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionId(pub usize);

/// One heading-delimited section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub id: SectionId,
    /// Heading level, 1..=6.
    pub level: u8,
    /// Heading text.
    pub title: String,
    /// The heading's source span.
    pub heading_span: Span,
    pub parent: Option<SectionId>,
    /// Child sections, in source order.
    pub children: Vec<SectionId>,
}

/// The B1 output: a flat section arena, the root forest, and a block→section
/// assignment.
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

/// Build the section model for a parsed document.
pub fn build_sections(_document: &Document) -> Sections {
    Sections::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_parser::parse;

    #[test]
    fn empty_document_has_no_sections() {
        let doc = parse("").unwrap();
        assert_eq!(build_sections(&doc), Sections::default());
    }
}
