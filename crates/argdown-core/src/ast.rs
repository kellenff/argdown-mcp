//! Argdown syntax-tree types.

use std::ops::Range;

/// A byte range into the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
        }
    }
}

/// A parsed Argdown document: a flat sequence of top-level blocks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// A top-level block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(Heading),
    Statement(Statement),
}

/// An ATX heading (`#`–`######`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub span: Span,
}

/// A statement: plain text, a titled definition (`[T]: x`), or a reference (`[T]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub title: Option<String>,
    pub text: String,
    pub is_reference: bool,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_from_range() {
        assert_eq!(Span::from(2..5), Span { start: 2, end: 5 });
    }

    #[test]
    fn document_default_is_empty() {
        assert_eq!(Document::default(), Document { blocks: vec![] });
    }
}
