//! Pure tool logic: `&str` source → plain result data. No protocol types.
//!
//! Shared by the `argdown-mcp` server and the `argdown` CLI. The optional
//! `schemars` feature adds `JsonSchema` derives to the result structs — that is
//! an MCP-adapter concern, off by default.

use argdown_core::Block;
use argdown_model::{
    analyze_af, build_model, dung_framework, grounded_extension, solve, AfMetadata, Algorithm,
    ArgumentId, Label, Model, Semantics, to_json, to_yaml,
};
use argdown_parser::parse;
use serde::Serialize;

/// A parse failure: human-readable message + byte offset into the source.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub offset: usize,
}

/// Syntactic block-kind counts for a successfully parsed document.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
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

/// Output serialization format for `model_export`.
#[derive(Debug, Clone, Copy)]
pub enum Format {
    Json,
    Yaml,
}

/// Why a tool could not produce its output.
#[derive(Debug)]
pub enum ToolError {
    /// The source did not parse.
    Parse(Diagnostic),
    /// The resolved model could not be serialized (e.g. non-string metadata key).
    Serialize(String),
}

/// Parse `source`, build the Layer B model, and serialize it in `format`.
pub fn model_export(source: &str, format: Format) -> Result<String, ToolError> {
    let doc = parse(source).map_err(|e| {
        ToolError::Parse(Diagnostic {
            message: e.message,
            offset: e.offset,
        })
    })?;
    let model = build_model(&doc);
    match format {
        Format::Json => to_json(&model).map_err(|e| ToolError::Serialize(e.to_string())),
        Format::Yaml => to_yaml(&model).map_err(|e| ToolError::Serialize(e.to_string())),
    }
}

/// A reference to an argument by its arena id and (optional) title.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct ArgRef {
    pub id: usize,
    pub title: Option<String>,
}

/// The grounded extension partition: accepted / defeated / undecided arguments.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
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

/// A projected AF argument with arena id and optional title.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct AfArgumentRef {
    pub id: usize,
    pub title: Option<String>,
}

/// A directed attack edge in the projected AF (`attacker` → `target`).
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct AfAttackRef {
    pub attacker: usize,
    pub target: usize,
}

/// Projected Dung AF plus structural metadata from SCC analysis.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct InspectAfResult {
    pub arguments: Vec<AfArgumentRef>,
    pub attacks: Vec<AfAttackRef>,
    #[cfg_attr(feature = "schemars", schemars(with = "schema::AfMetadataSchema"))]
    pub metadata: AfMetadata,
}

/// Parse `source`, project to a Dung AF, and return arguments, attacks, and
/// structural metadata.
pub fn inspect_af(source: &str) -> Result<InspectAfResult, Diagnostic> {
    let doc = parse(source).map_err(|e| Diagnostic {
        message: e.message,
        offset: e.offset,
    })?;
    let model = build_model(&doc);
    let af = dung_framework(&model);
    let metadata = analyze_af(&af);
    Ok(InspectAfResult {
        arguments: af
            .arguments
            .iter()
            .map(|&id| AfArgumentRef {
                id: id.0,
                title: title_for(&model, id),
            })
            .collect(),
        attacks: af
            .attacks
            .iter()
            .map(|&(from, to)| AfAttackRef {
                attacker: from.0,
                target: to.0,
            })
            .collect(),
        metadata,
    })
}

/// One argument's label in a reinstatement labelling.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct LabellingEntry {
    pub id: usize,
    pub title: Option<String>,
    /// `"in"`, `"out"`, or `"undec"`.
    pub label: String,
}

/// All labellings (or the unique labelling) for a Dung semantics.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct ExtensionsResult {
    #[cfg_attr(feature = "schemars", schemars(with = "schema::SemanticsSchema"))]
    pub semantics: Semantics,
    #[cfg_attr(feature = "schemars", schemars(with = "schema::AlgorithmSchema"))]
    pub algorithm: Algorithm,
    pub labellings: Vec<Vec<LabellingEntry>>,
    pub extension_sets: Vec<Vec<ArgRef>>,
}

/// Parse `source`, project to a Dung AF, and compute `semantics` extensions.
pub fn extensions(source: &str, semantics: Semantics) -> Result<ExtensionsResult, Diagnostic> {
    let doc = parse(source).map_err(|e| Diagnostic {
        message: e.message,
        offset: e.offset,
    })?;
    let model = build_model(&doc);
    let af = dung_framework(&model);
    let result = solve(&af, semantics);
    let labellings = result
        .labellings
        .iter()
        .map(|labeling| labeling_to_entries(&af, &model, labeling))
        .collect();
    let extension_sets = result
        .labellings
        .iter()
        .map(|labeling| in_extension(&af, &model, labeling))
        .collect();
    Ok(ExtensionsResult {
        semantics: result.semantics,
        algorithm: result.algorithm,
        labellings,
        extension_sets,
    })
}

fn title_for(model: &Model, id: ArgumentId) -> Option<String> {
    model.arguments.get(id.0).and_then(|a| a.title.clone())
}

fn label_str(label: Label) -> &'static str {
    match label {
        Label::In => "in",
        Label::Out => "out",
        Label::Undec => "undec",
    }
}

fn labeling_to_entries(
    af: &argdown_model::ArgumentationFramework,
    model: &Model,
    labeling: &argdown_model::Labeling,
) -> Vec<LabellingEntry> {
    af.arguments
        .iter()
        .map(|&id| LabellingEntry {
            id: id.0,
            title: title_for(model, id),
            label: label_str(labeling.get(&id).copied().unwrap_or(Label::Undec)).to_string(),
        })
        .collect()
}

fn in_extension(
    af: &argdown_model::ArgumentationFramework,
    model: &Model,
    labeling: &argdown_model::Labeling,
) -> Vec<ArgRef> {
    af.arguments
        .iter()
        .filter(|&&id| labeling.get(&id).copied() == Some(Label::In))
        .map(|&id| ArgRef {
            id: id.0,
            title: title_for(model, id),
        })
        .collect()
}

#[cfg(feature = "schemars")]
mod schema {
    use schemars::JsonSchema;

    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(remote = "argdown_model::AfMetadata")]
    pub struct AfMetadataSchema {
        argument_count: usize,
        attack_count: usize,
        is_acyclic: bool,
        has_self_attacks: bool,
        strongly_connected_components: Vec<Vec<usize>>,
        isolated_arguments: Vec<usize>,
    }

    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(remote = "argdown_model::Semantics")]
    #[serde(rename_all = "snake_case")]
    pub enum SemanticsSchema {
        Grounded,
        Preferred,
        Stable,
        Complete,
    }

    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(remote = "argdown_model::Algorithm")]
    #[serde(rename_all = "snake_case")]
    pub enum AlgorithmSchema {
        GroundedFixpoint,
        SccPropagationOnly,
        SccWithBacktracking,
        FilteredComplete,
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
        assert_eq!(
            s.blocks,
            s.headings + s.statements + s.arguments + s.relations + s.pcs
        );
        assert!(r.diagnostic.is_none());
    }

    #[test]
    fn summarize_reports_a_diagnostic_on_malformed_source() {
        let r = summarize("# H {unterminated");
        assert!(!r.ok);
        assert!(r.summary.is_none());
        let d = r.diagnostic.expect("diagnostic present on failure");
        assert!(d.offset <= "# H {unterminated".len());
        assert!(!d.message.is_empty());
    }

    #[test]
    fn model_export_json_serializes_the_resolved_model() {
        let json =
            model_export("<A>: d\n\n(1) P1\n----\n(2) C1", Format::Json).expect("valid model");
        let v: serde_json::Value = serde_json::from_str(&json).expect("reparses");
        let obj = v.as_object().expect("top-level object");
        for key in ["statements", "arguments", "pcs", "edges"] {
            assert!(obj.contains_key(key), "missing key {key}");
        }
        assert_eq!(v["pcs"][0]["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn model_export_returns_parse_error_with_offset() {
        let err = model_export("[A]: x { y", Format::Json).unwrap_err();
        match err {
            ToolError::Parse(d) => assert!(d.offset <= "[A]: x { y".len()),
            other => panic!("expected ToolError::Parse, got {other:?}"),
        }
    }

    #[test]
    fn model_export_yaml_round_trips_to_an_equal_model() {
        use argdown_model::from_yaml;
        let src = "<A>: a\n\n(1) P\n----\n(2) C";
        let yaml = model_export(src, Format::Yaml).expect("valid yaml");
        let back = from_yaml(&yaml).expect("valid round-trip");
        let model = build_model(&parse(src).unwrap());
        assert_eq!(back, model);
    }

    #[test]
    fn dung_partitions_a_simple_attack() {
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

    #[test]
    fn inspect_af_shows_attack_edge() {
        let r = inspect_af("<A>: a\n\n<B>: b\n  -> <A>").unwrap();
        assert_eq!(r.attacks.len(), 1);
        assert_eq!(r.attacks[0].attacker, 1);
    }

    #[test]
    fn extensions_grounded_matches_dung() {
        let old = dung("<A>: a\n\n<B>: b\n  -> <A>").unwrap();
        let new = extensions("<A>: a\n\n<B>: b\n  -> <A>", Semantics::Grounded).unwrap();
        assert_eq!(
            new.extension_sets[0]
                .iter()
                .map(|a| a.id)
                .collect::<Vec<_>>(),
            old.in_.iter().map(|a| a.id).collect::<Vec<_>>()
        );
    }
}
