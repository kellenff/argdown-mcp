//! Section assembly (Layer B, slice B1).
//!
//! Turns the flat parsed [`Document`] into a nested section tree plus a
//! block→section assignment. Pure and total: computed from `&Document`, the AST
//! is never mutated. Sections are a flat arena navigated by [`SectionId`].

use argdown_core::{Block, Document, Span};

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
///
/// Single pass over `document.blocks`, maintaining a stack of open sections
/// (increasing level) and the current (innermost) section. A heading pops all
/// open sections whose level is `>=` its own, nests under what remains, and
/// becomes current; every block — including the heading that opens a section —
/// is assigned to the current section (`None` before the first heading).
pub fn build_sections(document: &Document) -> Sections {
    let mut sections: Vec<Section> = Vec::new();
    let mut roots: Vec<SectionId> = Vec::new();
    let mut block_sections: Vec<Option<SectionId>> = Vec::with_capacity(document.blocks.len());
    let mut stack: Vec<SectionId> = Vec::new();
    let mut current: Option<SectionId> = None;

    for block in &document.blocks {
        if let Block::Heading(heading) = block {
            while let Some(&top) = stack.last() {
                if sections[top.0].level >= heading.level {
                    stack.pop();
                } else {
                    break;
                }
            }
            let parent = stack.last().copied();
            let id = SectionId(sections.len());
            sections.push(Section {
                id,
                level: heading.level,
                title: heading.text.clone(),
                heading_span: heading.span,
                parent,
                children: Vec::new(),
            });
            match parent {
                Some(p) => sections[p.0].children.push(id),
                None => roots.push(id),
            }
            stack.push(id);
            current = Some(id);
        }
        block_sections.push(current);
    }

    Sections {
        sections,
        roots,
        block_sections,
    }
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

    #[test]
    fn single_heading_owns_its_heading_and_following_block() {
        // Blocks: [Heading("Top"), Statement("[A]")]
        let doc = parse("# Top\n\n[A]: claim").unwrap();
        let s = build_sections(&doc);

        assert_eq!(s.sections.len(), 1);
        assert_eq!(s.roots, vec![SectionId(0)]);
        assert_eq!(s.sections[0].level, 1);
        assert_eq!(s.sections[0].title, "Top");
        assert_eq!(s.sections[0].parent, None);
        assert!(s.sections[0].children.is_empty());
        // Both the heading block and the following statement belong to section 0.
        assert_eq!(
            s.block_sections,
            vec![Some(SectionId(0)), Some(SectionId(0))]
        );
    }

    #[test]
    fn nested_headings_form_a_parent_child_tree() {
        // Blocks: [Heading L1, Heading L2, Heading L3]
        let doc = parse("# Top\n\n## Sub\n\n### Deep").unwrap();
        let s = build_sections(&doc);

        assert_eq!(s.sections.len(), 3);
        assert_eq!(s.roots, vec![SectionId(0)]);

        // Top (0): root, child Sub (1).
        assert_eq!(s.sections[0].parent, None);
        assert_eq!(s.sections[0].children, vec![SectionId(1)]);
        // Sub (1): parent Top, child Deep (2).
        assert_eq!(s.sections[1].parent, Some(SectionId(0)));
        assert_eq!(s.sections[1].children, vec![SectionId(2)]);
        // Deep (2): parent Sub, no children.
        assert_eq!(s.sections[2].parent, Some(SectionId(1)));
        assert!(s.sections[2].children.is_empty());

        assert_eq!(
            s.block_sections,
            vec![Some(SectionId(0)), Some(SectionId(1)), Some(SectionId(2))]
        );
    }
}
