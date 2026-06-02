//! Winnow-based parser for the Argdown format (increment A1: spine).
//!
//! Parses headings, plain and titled statements, and comments into an
//! [`argdown_core::Document`]. See the A1 spine design spec.

mod statement;
mod trivia;

use argdown_core::{Block, Document, Error};
use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{repeat, terminated};
use winnow::stream::LocatingSlice;

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
    statement.map(Block::Statement).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use argdown_core::{Span, Statement};

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
                    span: Span { start: 0, end: 12 },
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
                    span: Span { start: 0, end: 16 },
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
                    span: Span { start: 0, end: 17 },
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
                span: Span { start: 0, end: 4 },
            })]
        );
    }

    #[test]
    fn bare_bracket_without_colon_is_plain_text() {
        assert_eq!(
            parse("[Foo] is text").unwrap().blocks,
            vec![Block::Statement(Statement {
                title: None,
                text: "[Foo] is text".to_string(),
                span: Span { start: 0, end: 13 },
            })]
        );
    }
}
