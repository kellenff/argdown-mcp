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
    Argument(Argument),
    Relation(Relation),
    Pcs(Pcs),
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
    pub inlines: Vec<Inline>,
}

/// An argument: a titled definition (`<T>: desc`) or a reference (`<T>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    pub title: String,
    pub description: String,
    pub is_reference: bool,
    pub span: Span,
    pub inlines: Vec<Inline>,
}

/// One inline element inside a statement/argument body. `span` is the full
/// source extent of the element (opening delimiter through closing delimiter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inline {
    pub kind: InlineKind,
    pub span: Span,
}

/// The kind of an inline element, with its extracted data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineKind {
    Bold,
    Italic,
    Link { url: String },
    StatementMention { title: String },
    ArgumentMention { title: String },
    Tag { tag: String },
}

/// A relation line (`+`, `<+`, `+>`, `-`, `<-`, `->`, `_`, `<_`, `_>`, `><`)
/// and its target. The parser emits relations flat, in source order, tagged
/// with raw indentation; assembling the parent/child tree is Layer B's job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    /// Count of leading whitespace chars before the operator.
    pub indent: usize,
    pub operator: RelationOperator,
    pub direction: RelationDirection,
    pub target: RelationTarget,
    /// Operator start → target end (excludes the indent).
    pub span: Span,
}

/// The kind of dialectical relation, with the `+`≡`<+` family collapsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationOperator {
    Support,
    Attack,
    Undercut,
    Contradictory,
}

/// Direction relative to the implicit parent element (the less-indented line
/// above). `Inbound` = the relation points from the target to the parent
/// (`+`, `<+`, etc.). `Outbound` = from the parent to the target (`+>`, `->`,
/// `_>`). `Bidirectional` = `><`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

/// A relation's target: a statement or an argument, reusing those forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationTarget {
    Statement(Statement),
    Argument(Argument),
}

/// A premise-conclusion structure: a flat, source-order sequence of items.
/// Role assignment (premise/conclusion), inference→conclusion binding, and
/// relation association are Layer B's job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcs {
    pub items: Vec<PcsItem>,
    /// First item span start → last item span end.
    pub span: Span,
}

/// One line of a PCS, tagged by form (not role).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PcsItem {
    /// `(n) <statement>` — content reuses the statement forms.
    Statement {
        number: usize,
        statement: Statement,
        /// The `(` of the marker → statement content end.
        span: Span,
    },
    /// `----` (bare → empty rules) or `-- Rule, Rule --` (ruled).
    Inference { rules: Vec<String>, span: Span },
    /// An interspersed relation line, reusing the relation form (with indent).
    Relation(Relation),
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
