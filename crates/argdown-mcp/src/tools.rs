//! Pure tool logic: `&str` source → plain result data. No rmcp/protocol types.

use argdown_core::Block;
use argdown_model::{ArgumentId, build_model, dung_framework, grounded_extension, to_json};
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
            ParseResult {
                ok: true,
                summary: Some(summary),
                diagnostic: None,
            }
        }
        Err(e) => ParseResult {
            ok: false,
            summary: None,
            diagnostic: Some(Diagnostic {
                message: e.message,
                offset: e.offset,
            }),
        },
    }
}

/// Why a tool could not produce its output.
#[derive(Debug)]
pub enum ToolError {
    /// The source did not parse.
    Parse(Diagnostic),
    /// The resolved model could not be serialized (e.g. non-string metadata key).
    Serialize(String),
}

/// Parse `source`, build the Layer B model, and return it as pretty-printed JSON.
pub fn model_json(source: &str) -> Result<String, ToolError> {
    let doc = parse(source).map_err(|e| {
        ToolError::Parse(Diagnostic {
            message: e.message,
            offset: e.offset,
        })
    })?;
    let model = build_model(&doc);
    to_json(&model).map_err(|e| ToolError::Serialize(e.to_string()))
}

/// A reference to an argument by its arena id and (optional) title.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ArgRef {
    pub id: usize,
    pub title: Option<String>,
}

/// The grounded extension partition: accepted / defeated / undecided arguments.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DungResult {
    #[serde(rename = "in")]
    pub in_: Vec<ArgRef>,
    pub out: Vec<ArgRef>,
    pub undec: Vec<ArgRef>,
}

/// Parse `source`, build the model, project to a Dung AF, and return the
/// grounded extension with arguments resolved to `{id, title}`.
pub fn dung(source: &str) -> Result<DungResult, Diagnostic> {
    let doc = parse(source).map_err(|e| Diagnostic {
        message: e.message,
        offset: e.offset,
    })?;
    let model = build_model(&doc);
    let af = dung_framework(&model);
    let labelling = grounded_extension(&af);
    let to_refs = |ids: &[ArgumentId]| -> Vec<ArgRef> {
        ids.iter()
            .map(|id| ArgRef {
                id: id.0,
                title: model.arguments.get(id.0).and_then(|a| a.title.clone()),
            })
            .collect()
    };
    Ok(DungResult {
        in_: to_refs(&labelling.in_),
        out: to_refs(&labelling.out),
        undec: to_refs(&labelling.undec),
    })
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
        assert_eq!(
            s.blocks,
            s.headings + s.statements + s.arguments + s.relations + s.pcs
        );
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

    #[test]
    fn model_json_serializes_the_resolved_model() {
        let json = model_json("<A>: d\n\n(1) P1\n----\n(2) C1").expect("valid model");
        let v: serde_json::Value = serde_json::from_str(&json).expect("reparses");
        let obj = v.as_object().expect("top-level object");
        for key in ["statements", "arguments", "pcs", "edges"] {
            assert!(obj.contains_key(key), "missing key {key}");
        }
        assert_eq!(v["pcs"][0]["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn model_json_returns_parse_error_with_offset() {
        let err = model_json("[A]: x { y").unwrap_err();
        match err {
            ToolError::Parse(d) => assert!(d.offset <= "[A]: x { y".len()),
            other => panic!("expected ToolError::Parse, got {other:?}"),
        }
    }

    #[test]
    fn dung_partitions_a_simple_attack() {
        // <B> attacks <A>: B is unattacked (IN), A is defeated (OUT).
        let d = dung("<A>: a\n\n<B>: b\n  -> <A>").expect("valid");
        let titles = |refs: &[ArgRef]| {
            refs.iter()
                .filter_map(|a| a.title.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(titles(&d.in_), vec!["B"]);
        assert_eq!(titles(&d.out), vec!["A"]);
        assert!(d.undec.is_empty());
    }

    #[test]
    fn dung_returns_parse_error_with_offset() {
        let d = dung("[A]: x { y").unwrap_err();
        assert!(d.offset <= "[A]: x { y".len());
    }
}
