//! Tag registry (Layer B, slice B6a).
//!
//! Collects the inline `#tag`s used anywhere in a document into a registry of
//! unique tag names (in first-occurrence source order) plus a per-block
//! assignment. Tags are inline elements only (`InlineKind::Tag`) inside
//! statement/argument bodies; they appear at every site such a body appears —
//! top-level statements/arguments, the numbered statements of a PCS, and the
//! targets of relations. Pure and total. Metadata `tags:` promotion and
//! per-equivalence-class aggregation are deferred; the Dung map is B6b.

use argdown_core::{Block, Document, Inline, InlineKind, PcsItem, RelationTarget};
use std::collections::HashMap;

/// Stable, first-occurrence-order id; indexes `Tags::tags`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub usize);

/// The B6a output: the unique tags used in the document and their per-block
/// assignment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tags {
    /// Unique tag names in first-occurrence (source) order; `TagId(i)` indexes
    /// this.
    pub tags: Vec<String>,
    /// Index-aligned with `document.blocks`: the tags appearing anywhere within
    /// each top-level block (its own body, its PCS items, its relation
    /// targets), deduped, in source order.
    pub block_tags: Vec<Vec<TagId>>,
}

/// Build the tag registry for a parsed document.
///
/// Single pass over `document.blocks`, collecting `InlineKind::Tag` strings from
/// every statement/argument body each block contains; first occurrence assigns
/// the `TagId`, and each block records its own deduped tag list.
pub fn build_tags(document: &Document) -> Tags {
    let mut tags: Vec<String> = Vec::new();
    let mut by_name: HashMap<String, TagId> = HashMap::new();
    let mut block_tags: Vec<Vec<TagId>> = Vec::with_capacity(document.blocks.len());

    for block in &document.blocks {
        let mut ids: Vec<TagId> = Vec::new();
        match block {
            Block::Statement(s) => collect_inlines(&s.inlines, &mut tags, &mut by_name, &mut ids),
            Block::Argument(a) => collect_inlines(&a.inlines, &mut tags, &mut by_name, &mut ids),
            Block::Relation(r) => collect_target(&r.target, &mut tags, &mut by_name, &mut ids),
            Block::Pcs(p) => {
                for item in &p.items {
                    match item {
                        PcsItem::Statement { statement, .. } => {
                            collect_inlines(&statement.inlines, &mut tags, &mut by_name, &mut ids)
                        }
                        PcsItem::Relation(r) => {
                            collect_target(&r.target, &mut tags, &mut by_name, &mut ids)
                        }
                        PcsItem::Inference { .. } => {}
                    }
                }
            }
            Block::Heading(_) => {}
        }
        block_tags.push(ids);
    }

    Tags { tags, block_tags }
}

/// Register every `#tag` in `inlines`: assign a `TagId` on first occurrence
/// (extending the registry) and append it to this block's deduped list.
fn collect_inlines(
    inlines: &[Inline],
    tags: &mut Vec<String>,
    by_name: &mut HashMap<String, TagId>,
    ids: &mut Vec<TagId>,
) {
    for inline in inlines {
        if let InlineKind::Tag { tag } = &inline.kind {
            let id = match by_name.get(tag) {
                Some(&id) => id,
                None => {
                    let id = TagId(tags.len());
                    tags.push(tag.clone());
                    by_name.insert(tag.clone(), id);
                    id
                }
            };
            // Linear dedup: per-block tag counts are small.
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
}

/// A relation's target is a statement or argument; collect tags from its body.
fn collect_target(
    target: &RelationTarget,
    tags: &mut Vec<String>,
    by_name: &mut HashMap<String, TagId>,
    ids: &mut Vec<TagId>,
) {
    match target {
        RelationTarget::Statement(s) => collect_inlines(&s.inlines, tags, by_name, ids),
        RelationTarget::Argument(a) => collect_inlines(&a.inlines, tags, by_name, ids),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_parser::parse;

    #[test]
    fn inline_tags_on_a_statement_register_in_source_order() {
        let t = build_tags(&parse("[A]: a claim #foo #bar").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(t.block_tags, vec![vec![TagId(0), TagId(1)]]);
    }

    #[test]
    fn empty_document_has_no_tags() {
        let t = build_tags(&parse("").unwrap());
        assert_eq!(t, Tags::default());
    }

    #[test]
    fn tags_on_an_argument_are_collected() {
        let t = build_tags(&parse("<A>: an arg #foo").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string()]);
        assert_eq!(t.block_tags, vec![vec![TagId(0)]]);
    }

    #[test]
    fn registry_is_first_occurrence_order_across_blocks() {
        let t = build_tags(&parse("[A]: a #foo\n\n[B]: b #bar #foo").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string(), "bar".to_string()]);
        // Block 1 re-uses foo's id and orders by its own source order: bar, foo.
        assert_eq!(t.block_tags, vec![vec![TagId(0)], vec![TagId(1), TagId(0)]]);
    }

    #[test]
    fn a_tag_repeated_in_one_block_is_deduped() {
        let t = build_tags(&parse("[A]: a #foo #foo").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string()]);
        assert_eq!(t.block_tags, vec![vec![TagId(0)]]);
    }

    #[test]
    fn tag_inside_a_pcs_numbered_statement_is_collected() {
        let t = build_tags(&parse("(1) [P]: p #foo\n----\n(2) C").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string()]);
        // One PCS block carries the tag.
        assert_eq!(t.block_tags, vec![vec![TagId(0)]]);
    }

    #[test]
    fn tag_on_a_relation_target_is_collected() {
        let t = build_tags(&parse("[A]: a\n  + [B]: b #foo").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string()]);
        // Block 0 = [A] (no tags); block 1 = the relation (target [B] tagged).
        assert_eq!(t.block_tags, vec![vec![], vec![TagId(0)]]);
    }

    #[test]
    fn tag_on_an_interspersed_pcs_relation_target_is_collected() {
        // The other PCS sub-arm: a relation interspersed in a PCS, its target tagged.
        let t = build_tags(&parse("(1) P\n  +> [S]: s #foo\n----\n(2) C").unwrap());
        assert_eq!(t.tags, vec!["foo".to_string()]);
        assert_eq!(t.block_tags, vec![vec![TagId(0)]]);
    }

    #[test]
    fn parenthesized_multi_word_tag_is_registered_verbatim() {
        let t = build_tags(&parse("[A]: a #(multi word)").unwrap());
        assert_eq!(t.tags, vec!["multi word".to_string()]);
        assert_eq!(t.block_tags, vec![vec![TagId(0)]]);
    }

    #[test]
    fn no_tags_yields_empty_registry_and_empty_per_block() {
        let t = build_tags(&parse("[A]: a plain claim\n\n<B>: an arg").unwrap());
        assert!(t.tags.is_empty());
        assert_eq!(t.block_tags, vec![vec![], vec![]]);
    }

    #[test]
    fn block_tags_is_index_aligned_with_blocks() {
        let doc = parse("# H\n\n[A]: a #foo\n\n<B>: b").unwrap();
        let t = build_tags(&doc);
        assert_eq!(t.block_tags.len(), doc.blocks.len());
    }
}
