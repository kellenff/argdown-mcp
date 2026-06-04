//! Winnow-based parser for the Argdown format.
//!
//! Parses the document spine (headings, statements, comments), statement and
//! argument definitions, and references into an [`argdown_core::Document`].
//! Grows by increment.

mod argument;
mod heading;
mod inline;
mod pcs;
mod relation;
mod statement;
mod text;
mod trivia;

use argdown_core::{Block, Document, Error};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{alt, repeat, terminated};
use winnow::stream::LocatingSlice;

use argument::argument;
use heading::heading;
use pcs::pcs;
use relation::relation;
use statement::statement;
use trivia::skip_trivia;

/// The winnow input stream: `&str` augmented with byte-offset locations.
pub(crate) type Input<'s> = LocatingSlice<&'s str>;

/// Parse Argdown source text into a [`Document`].
pub fn parse(source: &str) -> Result<Document, Error> {
    document.parse(Input::new(source)).map_err(|e| Error {
        message: e.to_string(),
        offset: e.offset(),
    })
}

fn document(input: &mut Input<'_>) -> ModalResult<Document> {
    skip_trivia(input)?;
    let blocks: Vec<Block> = repeat(0.., terminated(block, skip_trivia)).parse_next(input)?;
    Ok(Document { blocks })
}

fn block(input: &mut Input<'_>) -> ModalResult<Block> {
    alt((
        heading.map(Block::Heading),
        relation.map(Block::Relation),
        pcs.map(Block::Pcs),
        argument.map(Block::Argument),
        statement.map(Block::Statement),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_core::{
        Argument, Heading, Inline, InlineKind, Pcs, PcsItem, Relation, RelationDirection,
        RelationOperator, RelationTarget, Span, Statement,
    };

    /// The single statement a source parses to, panicking otherwise.
    fn only_statement(src: &str) -> Statement {
        match parse(src).unwrap().blocks.as_slice() {
            [Block::Statement(s)] => s.clone(),
            other => panic!("{src:?} did not parse as a single statement: {other:?}"),
        }
    }

    #[test]
    fn inline_italic_and_bold_plain_statement() {
        let s = only_statement("this is *it* and **bold**");
        assert_eq!(
            s.inlines,
            vec![
                Inline {
                    kind: InlineKind::Italic,
                    span: Span { start: 8, end: 12 }
                },
                Inline {
                    kind: InlineKind::Bold,
                    span: Span { start: 17, end: 25 }
                },
            ]
        );
    }

    #[test]
    fn inline_underscore_emphasis() {
        let s = only_statement("_i_ and __b__");
        assert_eq!(s.inlines[0].kind, InlineKind::Italic);
        assert_eq!(s.inlines[1].kind, InlineKind::Bold);
    }

    #[test]
    fn inline_emphasis_nests_as_contained_spans() {
        let s = only_statement("**bold and *italic* inside**");
        // Bold first (source order by start), then the contained italic.
        assert_eq!(s.inlines[0].kind, InlineKind::Bold);
        assert_eq!(s.inlines[1].kind, InlineKind::Italic);
        let (b, i) = (s.inlines[0].span, s.inlines[1].span);
        assert!(
            b.start <= i.start && i.end <= b.end,
            "italic must be contained in bold"
        );
    }

    #[test]
    fn inline_link_with_url() {
        let s = only_statement("see [the site](http://x.com) now");
        assert_eq!(s.inlines.len(), 1);
        match &s.inlines[0].kind {
            InlineKind::Link { url } => assert_eq!(url, "http://x.com"),
            other => panic!("expected a link, got {other:?}"),
        }
        // Span covers the whole `[the site](http://x.com)`.
        assert_eq!(s.inlines[0].span, Span { start: 4, end: 28 });
    }

    #[test]
    fn bracket_without_paren_is_literal() {
        let s = only_statement("note [1] applies");
        assert!(s.inlines.is_empty());
    }

    #[test]
    fn inline_statement_mention() {
        let s = only_statement("recall @[Other Claim] here");
        match &s.inlines[0].kind {
            InlineKind::StatementMention { title } => assert_eq!(title, "Other Claim"),
            other => panic!("expected a statement mention, got {other:?}"),
        }
    }

    #[test]
    fn inline_argument_mention() {
        let s = only_statement("per @<Some Arg> there");
        match &s.inlines[0].kind {
            InlineKind::ArgumentMention { title } => assert_eq!(title, "Some Arg"),
            other => panic!("expected an argument mention, got {other:?}"),
        }
    }

    #[test]
    fn bare_at_is_literal() {
        let s = only_statement("email a@b.com please");
        assert!(s.inlines.is_empty());
    }

    #[test]
    fn inline_contiguous_tag() {
        let s = only_statement("flagged #simple-tag here");
        match &s.inlines[0].kind {
            InlineKind::Tag { tag } => assert_eq!(tag, "simple-tag"),
            other => panic!("expected a tag, got {other:?}"),
        }
    }

    #[test]
    fn inline_parenthesized_tag() {
        let s = only_statement("flagged #(multi word) here");
        match &s.inlines[0].kind {
            InlineKind::Tag { tag } => assert_eq!(tag, "multi word"),
            other => panic!("expected a tag, got {other:?}"),
        }
    }

    #[test]
    fn bare_hash_is_literal() {
        let s = only_statement("rooms # and # are free");
        assert!(s.inlines.is_empty());
    }

    #[test]
    fn inline_escape_suppresses_emphasis() {
        let s = only_statement(r"this \*is not\* italic");
        assert!(s.inlines.is_empty());
    }

    #[test]
    fn prose_with_stray_delimiters_stays_literal() {
        for src in [
            "cost is 5 * 3 dollars",
            "use snake_case names",
            "item # 4 here",
        ] {
            let s = only_statement(src);
            assert!(s.inlines.is_empty(), "{src:?} should have no inlines");
        }
    }

    #[test]
    fn unclosed_emphasis_is_an_error() {
        assert!(parse("this is **bold with no close").is_err());
    }

    #[test]
    fn link_without_closing_paren_is_an_error() {
        assert!(parse("see [text](http://x.com here").is_err());
    }

    #[test]
    fn parenthesized_tag_without_close_is_an_error() {
        assert!(parse("flagged #(multi word here").is_err());
    }

    #[test]
    fn no_space_after_underscore_is_italic_statement_not_undercut() {
        // `_emphasis_` (no space after the `_`) is an italic statement, while
        // `+ [B]` (space after the operator) is still a relation. The trailing
        // space is the grammar boundary between an emphasis line and a relation.
        let s = only_statement("_emphasis_ here");
        assert_eq!(s.inlines[0].kind, InlineKind::Italic);
        let r = only_relation("+ [B]");
        assert_eq!(r.operator, RelationOperator::Support);
    }

    #[test]
    fn parse_empty_input_yields_empty_document() {
        assert_eq!(parse(""), Ok(Document::default()));
    }

    #[test]
    fn single_plain_statement() {
        assert_eq!(
            parse("Hello world."),
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: None,
                    text: "Hello world.".to_string(),
                    is_reference: false,
                    span: Span { start: 0, end: 12 },
                    inlines: vec![],
                })],
            })
        );
    }

    #[test]
    fn titled_statement() {
        assert_eq!(
            parse("[Key]: Some text"),
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: Some("Key".to_string()),
                    text: "Some text".to_string(),
                    is_reference: false,
                    span: Span { start: 0, end: 16 },
                    inlines: vec![],
                })],
            })
        );
    }

    #[test]
    fn multi_line_statement_is_normalized() {
        assert_eq!(
            parse("Line one\nline two"),
            Ok(Document {
                blocks: vec![Block::Statement(Statement {
                    title: None,
                    text: "Line one line two".to_string(),
                    is_reference: false,
                    span: Span { start: 0, end: 17 },
                    inlines: vec![],
                })],
            })
        );
    }

    #[test]
    fn blank_line_separates_statements() {
        let doc = parse("a\n\nb").unwrap();
        assert_eq!(doc.blocks.len(), 2);
    }

    #[test]
    fn crlf_within_statement() {
        assert_eq!(
            parse("a\r\nb").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "a b".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 4 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn text_after_statement_reference_is_an_error() {
        // `[Foo] is text` — what A1 treated as plain text is now an error.
        let err = parse("[Foo] is text").unwrap_err();
        assert_eq!(err.offset, 6);
    }

    #[test]
    fn statement_reference() {
        assert_eq!(
            parse("[S]").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: Some("S".to_string()),
                text: String::new(),
                is_reference: true,
                span: Span { start: 0, end: 3 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn statement_definition_still_parses() {
        assert_eq!(
            parse("[S]: text").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: Some("S".to_string()),
                text: "text".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 9 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn two_references_on_adjacent_lines() {
        let blocks = parse("[A]\n[B]").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Statement(s) if s.is_reference));
        assert!(matches!(&blocks[1], Block::Statement(s) if s.is_reference));
    }

    #[test]
    fn text_after_reference_same_line_offset() {
        assert_eq!(parse("[S] words").unwrap_err().offset, 4);
    }

    #[test]
    fn text_after_reference_next_line_offset() {
        assert_eq!(parse("[S]\nwords").unwrap_err().offset, 4);
    }

    #[test]
    fn heading_level_one() {
        assert_eq!(
            parse("# Title").unwrap().blocks,
            vec![Block::Heading(Heading {
                level: 1,
                text: "Title".to_string(),
                span: Span { start: 0, end: 7 },
            })]
        );
    }

    #[test]
    fn heading_levels_two_through_six() {
        for level in 2u8..=6 {
            let hashes = "#".repeat(level as usize);
            let source = format!("{hashes} Deep");
            let blocks = parse(&source).unwrap().blocks;
            assert_eq!(
                blocks,
                vec![Block::Heading(Heading {
                    level,
                    text: "Deep".to_string(),
                    span: Span {
                        start: 0,
                        end: source.len(),
                    },
                })]
            );
        }
    }

    #[test]
    fn heading_then_statement_without_blank_line() {
        let blocks = parse("# Title\nbody").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0], Block::Heading(_)));
        assert!(matches!(blocks[1], Block::Statement(_)));
    }

    #[test]
    fn hash_without_space_is_a_statement() {
        let blocks = parse("#nospace").unwrap().blocks;
        assert_eq!(
            blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "#nospace".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 8 },
                inlines: vec![Inline {
                    kind: InlineKind::Tag {
                        tag: "nospace".to_string()
                    },
                    span: Span { start: 0, end: 8 },
                }],
            })]
        );
    }

    #[test]
    fn line_comment_between_statements_is_skipped() {
        let blocks = parse("a\n// note\nb").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn trailing_line_comment_is_stripped() {
        assert_eq!(
            parse("foo // bar").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "foo".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 10 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn block_comment_spanning_lines_is_skipped() {
        let blocks = parse("a\n/* x\ny */\nb").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn html_comment_is_skipped() {
        let blocks = parse("<!-- c -->\nb").unwrap().blocks;
        assert_eq!(
            blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "b".to_string(),
                is_reference: false,
                span: Span { start: 11, end: 12 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn unterminated_block_comment_errors_at_opener() {
        let err = parse("/* oops").unwrap_err();
        assert_eq!(err.offset, 0);
    }

    #[test]
    fn error_offset_points_past_earlier_blocks() {
        let err = parse("foo\n/* x").unwrap_err();
        assert_eq!(err.offset, 4);
    }

    #[test]
    fn argument_definition_single_line() {
        assert_eq!(
            parse("<A>: desc").unwrap().blocks,
            vec![Block::Argument(Argument {
                title: "A".to_string(),
                description: "desc".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 9 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn argument_definition_multi_line() {
        assert_eq!(
            parse("<A>: one\ntwo").unwrap().blocks,
            vec![Block::Argument(Argument {
                title: "A".to_string(),
                description: "one two".to_string(),
                is_reference: false,
                span: Span { start: 0, end: 12 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn argument_reference() {
        assert_eq!(
            parse("<A>").unwrap().blocks,
            vec![Block::Argument(Argument {
                title: "A".to_string(),
                description: String::new(),
                is_reference: true,
                span: Span { start: 0, end: 3 },
                inlines: vec![],
            })]
        );
    }

    #[test]
    fn text_after_argument_reference_is_an_error() {
        assert_eq!(parse("<A> words").unwrap_err().offset, 4);
        assert_eq!(parse("<A>\nwords").unwrap_err().offset, 4);
    }

    /// Extract the single relation a source parses to, panicking otherwise.
    fn only_relation(src: &str) -> Relation {
        match parse(src).unwrap().blocks.as_slice() {
            [Block::Relation(r)] => r.clone(),
            other => panic!("{src:?} did not parse as a single relation: {other:?}"),
        }
    }

    #[test]
    fn operator_tokens_map_to_operator_and_direction() {
        use RelationDirection::{Bidirectional, Inbound, Outbound};
        use RelationOperator::{Attack, Contradictory, Support, Undercut};
        let cases = [
            ("+ [B]", Support, Inbound),
            ("<+ [B]", Support, Inbound),
            ("+> [B]", Support, Outbound),
            ("- [B]", Attack, Inbound),
            ("<- [B]", Attack, Inbound),
            ("-> [B]", Attack, Outbound),
            ("_ [B]", Undercut, Inbound),
            ("<_ [B]", Undercut, Inbound),
            ("_> [B]", Undercut, Outbound),
            (">< [B]", Contradictory, Bidirectional),
        ];
        for (src, operator, direction) in cases {
            let r = only_relation(src);
            assert_eq!(r.operator, operator, "operator for {src:?}");
            assert_eq!(r.direction, direction, "direction for {src:?}");
        }
    }

    #[test]
    fn relation_under_a_reference_is_not_text_after_reference() {
        let blocks = parse("[A]\n  + [B]").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(
            matches!(&blocks[0], Block::Statement(s) if s.is_reference && s.title.as_deref() == Some("A"))
        );
        assert!(matches!(&blocks[1], Block::Relation(r) if r.indent == 2));
    }

    #[test]
    fn indentation_is_captured_as_leading_whitespace_count() {
        assert_eq!(only_relation("  + [B]").indent, 2);
        assert_eq!(only_relation("    - [C]").indent, 4);
    }

    #[test]
    fn support_relation_to_statement_reference() {
        assert_eq!(
            parse("+ [B]").unwrap().blocks,
            vec![Block::Relation(Relation {
                indent: 0,
                operator: RelationOperator::Support,
                direction: RelationDirection::Inbound,
                target: RelationTarget::Statement(Statement {
                    title: Some("B".to_string()),
                    text: String::new(),
                    is_reference: true,
                    span: Span { start: 2, end: 5 },
                    inlines: vec![],
                }),
                span: Span { start: 0, end: 5 },
            })]
        );
    }

    #[test]
    fn nested_relations_are_flat_in_source_order_with_depth() {
        let blocks = parse("[A]\n  + [B]\n    - [C]\n  + [D]").unwrap().blocks;
        assert_eq!(blocks.len(), 4);
        assert!(matches!(&blocks[0], Block::Statement(s) if s.title.as_deref() == Some("A")));

        let expected = [
            (2, RelationOperator::Support, "B"),
            (4, RelationOperator::Attack, "C"),
            (2, RelationOperator::Support, "D"),
        ];
        for (block, (indent, operator, title)) in blocks[1..].iter().zip(expected) {
            let Block::Relation(r) = block else {
                panic!("expected a relation, got {block:?}");
            };
            assert_eq!(r.indent, indent);
            assert_eq!(r.operator, operator);
            assert_eq!(r.direction, RelationDirection::Inbound);
            match &r.target {
                RelationTarget::Statement(s) => assert_eq!(s.title.as_deref(), Some(title)),
                other => panic!("expected a statement target, got {other:?}"),
            }
        }
    }

    #[test]
    fn relation_target_is_a_statement_definition() {
        let r = only_relation("+ [B]: x");
        match r.target {
            RelationTarget::Statement(s) => {
                assert_eq!(s.title.as_deref(), Some("B"));
                assert_eq!(s.text, "x");
                assert!(!s.is_reference);
            }
            other => panic!("expected a statement target, got {other:?}"),
        }
    }

    #[test]
    fn relation_target_is_a_plain_statement() {
        let r = only_relation("+ claim");
        match r.target {
            RelationTarget::Statement(s) => {
                assert_eq!(s.title, None);
                assert_eq!(s.text, "claim");
            }
            other => panic!("expected a statement target, got {other:?}"),
        }
    }

    #[test]
    fn relation_target_is_an_argument_reference() {
        let r = only_relation("+ <Arg>");
        match r.target {
            RelationTarget::Argument(a) => {
                assert_eq!(a.title, "Arg");
                assert!(a.is_reference);
            }
            other => panic!("expected an argument target, got {other:?}"),
        }
    }

    #[test]
    fn relation_target_is_an_argument_definition() {
        let r = only_relation("+ <Arg>: desc");
        match r.target {
            RelationTarget::Argument(a) => {
                assert_eq!(a.title, "Arg");
                assert_eq!(a.description, "desc");
                assert!(!a.is_reference);
            }
            other => panic!("expected an argument target, got {other:?}"),
        }
    }

    #[test]
    fn multi_line_relation_target_is_normalized() {
        let r = only_relation("+ [B]: one\n    two");
        match r.target {
            RelationTarget::Statement(s) => assert_eq!(s.text, "one two"),
            other => panic!("expected a statement target, got {other:?}"),
        }
    }

    #[test]
    fn text_after_a_relation_target_reference_is_an_error() {
        assert!(parse("+ [B] extra").is_err());
    }

    #[test]
    fn relation_operator_without_a_target_is_an_error() {
        assert!(parse("+ ").is_err());
    }

    /// Extract the single PCS a source parses to, panicking otherwise.
    fn only_pcs(src: &str) -> Pcs {
        match parse(src).unwrap().blocks.as_slice() {
            [Block::Pcs(p)] => p.clone(),
            other => panic!("{src:?} did not parse as a single PCS: {other:?}"),
        }
    }

    #[test]
    fn pcs_single_numbered_statement() {
        let pcs = only_pcs("(1) a");
        assert_eq!(pcs.items.len(), 1);
        match &pcs.items[0] {
            PcsItem::Statement {
                number, statement, ..
            } => {
                assert_eq!(*number, 1);
                assert_eq!(statement.text, "a");
                assert_eq!(statement.title, None);
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn pcs_two_numbered_statements() {
        let pcs = only_pcs("(1) a\n(2) b");
        let numbers: Vec<usize> = pcs
            .items
            .iter()
            .map(|item| match item {
                PcsItem::Statement { number, .. } => *number,
                other => panic!("expected statement items, got {other:?}"),
            })
            .collect();
        assert_eq!(numbers, vec![1, 2]);
    }

    #[test]
    fn pcs_bare_inference_line() {
        let pcs = only_pcs("(1) a\n(2) b\n----\n(3) c");
        assert_eq!(pcs.items.len(), 4);
        match &pcs.items[2] {
            PcsItem::Inference { rules, .. } => assert!(rules.is_empty()),
            other => panic!("expected an inference item at index 2, got {other:?}"),
        }
        assert!(matches!(
            &pcs.items[1],
            PcsItem::Statement { number: 2, .. }
        ));
        assert!(matches!(
            &pcs.items[3],
            PcsItem::Statement { number: 3, .. }
        ));
    }

    #[test]
    fn pcs_bare_divider_allows_five_or_more_dashes() {
        let pcs = only_pcs("(1) a\n-----\n(2) b");
        assert!(matches!(&pcs.items[1], PcsItem::Inference { rules, .. } if rules.is_empty()));
    }

    #[test]
    fn pcs_three_dash_divider_is_an_error() {
        assert!(parse("(1) a\n---\n(2) b").is_err());
    }

    fn inference_rules_of(src: &str, index: usize) -> Vec<String> {
        match &only_pcs(src).items[index] {
            PcsItem::Inference { rules, .. } => rules.clone(),
            other => panic!("expected an inference item at {index}, got {other:?}"),
        }
    }

    #[test]
    fn pcs_ruled_inference_single_rule() {
        assert_eq!(
            inference_rules_of("(1) a\n-- Modus Ponens --\n(2) b", 1),
            vec!["Modus Ponens".to_string()]
        );
    }

    #[test]
    fn pcs_ruled_inference_multiple_rules() {
        assert_eq!(
            inference_rules_of("(1) a\n-- Rule A, Rule B --\n(2) b", 1),
            vec!["Rule A".to_string(), "Rule B".to_string()]
        );
    }

    #[test]
    fn pcs_ruled_inference_without_closing_dashes_is_an_error() {
        assert!(parse("(1) a\n-- Modus Ponens\n(2) b").is_err());
    }

    #[test]
    fn pcs_numbered_statement_spans_continuation_lines() {
        let pcs = only_pcs("(1) one\n    two");
        assert_eq!(pcs.items.len(), 1);
        match &pcs.items[0] {
            PcsItem::Statement { statement, .. } => assert_eq!(statement.text, "one two"),
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn pcs_marker_ends_a_previous_statements_continuation() {
        // Without the guard, `(2) b` would be swallowed as continuation text of (1).
        let pcs = only_pcs("(1) one\ntwo\n(2) b");
        match &pcs.items[0] {
            PcsItem::Statement {
                number, statement, ..
            } => {
                assert_eq!(*number, 1);
                assert_eq!(statement.text, "one two");
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
        assert!(matches!(
            &pcs.items[1],
            PcsItem::Statement { number: 2, .. }
        ));
    }

    #[test]
    fn pcs_ends_at_heading_and_reference() {
        let blocks = parse("(1) a\n# H").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Pcs(_)));
        assert!(matches!(&blocks[1], Block::Heading(_)));

        let blocks = parse("(1) a\n[X]").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Pcs(_)));
        assert!(matches!(&blocks[1], Block::Statement(s) if s.is_reference));
    }

    #[test]
    fn blank_line_separates_two_pcs_blocks() {
        let blocks = parse("(1) a\n\n(2) b").unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], Block::Pcs(p) if p.items.len() == 1));
        assert!(matches!(&blocks[1], Block::Pcs(p) if p.items.len() == 1));
    }

    #[test]
    fn pcs_interspersed_child_relation() {
        let pcs = only_pcs("(1) a\n  +> [X]\n----\n(2) b");
        assert_eq!(pcs.items.len(), 4);
        match &pcs.items[1] {
            PcsItem::Relation(relation) => {
                assert_eq!(relation.indent, 2);
                assert_eq!(relation.operator, RelationOperator::Support);
                assert_eq!(relation.direction, RelationDirection::Outbound);
                match &relation.target {
                    RelationTarget::Statement(s) => assert_eq!(s.title.as_deref(), Some("X")),
                    other => panic!("expected a statement target, got {other:?}"),
                }
            }
            other => panic!("expected a relation item at index 1, got {other:?}"),
        }
        assert!(matches!(&pcs.items[2], PcsItem::Inference { .. }));
    }

    #[test]
    fn pcs_numbered_statement_target_forms() {
        // Definition target.
        match &only_pcs("(1) [P]: text").items[0] {
            PcsItem::Statement { statement, .. } => {
                assert_eq!(statement.title.as_deref(), Some("P"));
                assert_eq!(statement.text, "text");
                assert!(!statement.is_reference);
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
        // Reference target.
        match &only_pcs("(1) [P]").items[0] {
            PcsItem::Statement { statement, .. } => {
                assert_eq!(statement.title.as_deref(), Some("P"));
                assert!(statement.is_reference);
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
        // Plain target.
        match &only_pcs("(1) plain").items[0] {
            PcsItem::Statement { statement, .. } => {
                assert_eq!(statement.title, None);
                assert_eq!(statement.text, "plain");
            }
            other => panic!("expected a statement item, got {other:?}"),
        }
    }

    #[test]
    fn pcs_numbered_marker_without_content_is_an_error() {
        // The marker commits; an empty body is a hard error, not a plain statement.
        assert!(parse("(1) a\n(2)").is_err());
    }

    #[test]
    fn pcs_text_after_reference_target_is_an_error() {
        assert!(parse("(1) [P] extra").is_err());
    }

    #[test]
    fn parenthesized_non_number_is_a_plain_statement() {
        // `(see note)` is not a numbered marker — it stays a plain statement.
        let blocks = parse("(see note)").unwrap().blocks;
        assert!(matches!(&blocks[0], Block::Statement(s) if s.text == "(see note)"));
    }

    #[test]
    fn pcs_multi_step_interleaved() {
        // premises -> bare inference -> intermediary -> premise -> ruled inference -> main
        let pcs = only_pcs("(1) a\n(2) b\n----\n(3) c\n(4) d\n-- R --\n(5) e");
        assert_eq!(pcs.items.len(), 7);
        assert!(matches!(&pcs.items[2], PcsItem::Inference { rules, .. } if rules.is_empty()));
        assert_eq!(
            inference_rules_of("(1) a\n(2) b\n----\n(3) c\n(4) d\n-- R --\n(5) e", 5),
            vec!["R".to_string()]
        );
        let numbers: Vec<usize> = pcs
            .items
            .iter()
            .filter_map(|item| match item {
                PcsItem::Statement { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(numbers, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn inline_in_argument_description() {
        let blocks = parse("<A>: this has **bold** text").unwrap().blocks;
        match &blocks[0] {
            Block::Argument(a) => assert_eq!(a.inlines[0].kind, InlineKind::Bold),
            other => panic!("expected an argument, got {other:?}"),
        }
    }

    #[test]
    fn inline_in_pcs_numbered_statement() {
        let blocks = parse("(1) a claim with *emphasis*").unwrap().blocks;
        match &blocks[0] {
            Block::Pcs(p) => match &p.items[0] {
                PcsItem::Statement { statement, .. } => {
                    assert_eq!(statement.inlines[0].kind, InlineKind::Italic);
                }
                other => panic!("expected a statement item, got {other:?}"),
            },
            other => panic!("expected a PCS, got {other:?}"),
        }
    }

    #[test]
    fn inline_span_absolute_across_a_definition_title() {
        // `[T]: ` is 5 bytes, so the bold opener `**` starts at byte 5.
        let s = only_statement("[T]: **b**");
        assert_eq!(s.inlines[0].span, Span { start: 5, end: 10 });
    }
}
