//! Pure tool logic: `&str` source → plain result data. No rmcp/protocol types.

use argdown_core::Block;
use argdown_parser::parse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Inline source input shared by every tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SourceInput {
    /// The Argdown source text to analyze.
    pub source: String,
}

/// A parse failure: human-readable message + byte offset into the source.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Diagnostic {
    pub message: String,
    pub offset: usize,
}

/// Syntactic block-kind counts for a successfully parsed document.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ParseSummary {
    pub blocks: usize,
    pub headings: usize,
    pub statements: usize,
    pub arguments: usize,
    pub relations: usize,
    pub pcs: usize,
    pub has_frontmatter: bool,
}

/// `parse` result: a summary on success, a diagnostic on failure. Never an error.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ParseResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ParseSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<Diagnostic>,
}

/// Parse `source` and report a syntactic summary, or a diagnostic on failure.
pub fn summarize(source: &str) -> ParseResult {
    match parse(source) {
        Ok(doc) => {
            let mut summary = ParseSummary {
                blocks: doc.blocks.len(),
                headings: 0,
                statements: 0,
                arguments: 0,
                relations: 0,
                pcs: 0,
                has_frontmatter: doc.frontmatter.is_some(),
            };
            for block in &doc.blocks {
                match block {
                    Block::Heading(_) => summary.headings += 1,
                    Block::Statement(_) => summary.statements += 1,
                    Block::Argument(_) => summary.arguments += 1,
                    Block::Relation(_) => summary.relations += 1,
                    Block::Pcs(_) => summary.pcs += 1,
                }
            }
            ParseResult { ok: true, summary: Some(summary), diagnostic: None }
        }
        Err(e) => ParseResult {
            ok: false,
            summary: None,
            diagnostic: Some(Diagnostic { message: e.message, offset: e.offset }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_counts_blocks_and_frontmatter() {
        let src = "===\ntitle: T\n===\n\n# H\n\n[S]: s\n\n<A>: a\n\n(1) P\n----\n(2) C";
        let r = summarize(src);
        assert!(r.ok);
        let s = r.summary.expect("summary present on success");
        assert!(s.has_frontmatter);
        assert_eq!(s.headings, 1);
        assert_eq!(s.statements, 1);
        assert_eq!(s.arguments, 1);
        assert_eq!(s.pcs, 1);
        assert_eq!(s.blocks, s.headings + s.statements + s.arguments + s.relations + s.pcs);
        assert!(r.diagnostic.is_none());
    }

    #[test]
    fn summarize_reports_a_diagnostic_on_malformed_source() {
        // An unterminated metadata block is a parse error.
        let r = summarize("# H {unterminated");
        assert!(!r.ok);
        assert!(r.summary.is_none());
        let d = r.diagnostic.expect("diagnostic present on failure");
        assert!(d.offset <= "# H {unterminated".len());
        assert!(!d.message.is_empty());
    }
}
