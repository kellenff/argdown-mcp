//! Pure tool logic: `&str` source → plain result data. No protocol types.
//!
//! Shared by the `argdown-mcp` server and the `argdown` CLI. The optional
//! `schemars` feature adds `JsonSchema` derives to the result structs — that is
//! an MCP-adapter concern, off by default.

use argdown_core::Block;
use argdown_model::{
    AfMetadata, Algorithm, ArgumentId, DEFAULT_MAX_ITERATIONS, Label, Model, Semantics, analyze_af,
    build_model, classify_degree, df_quad_run, dung_framework, grounded_extension, project_qbaf,
    solve, to_json, to_yaml,
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

/// One argument's DF-QuAD degree and threshold-based status.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct QbafDegree {
    pub id: usize,
    pub title: Option<String>,
    pub base: f64,
    pub final_degree: f64,
    /// `"accepted"`, `"rejected"`, or `"undec"` (non-convergence).
    pub status: String,
}

/// DF-QuAD evaluation result (object root for MCP output schema).
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct QbafEvaluateResult {
    pub semantics: String,
    pub threshold: f64,
    pub degrees: Vec<QbafDegree>,
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

/// Credulous vs skeptical acceptance mode for point queries.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceMode {
    Credulous,
    Skeptical,
}

/// Structured witness explaining why an argument is or is not accepted.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessType {
    AcceptedUncontroversial,
    AttackedByAccepted,
    UnsupportedCycle,
    MultipleInterpretations,
    SkepticallyRejected,
    UndefinedArgument,
}

/// Evidence backing an [`AcceptsResult`].
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct WitnessPayload {
    #[serde(rename = "type")]
    pub witness_type: WitnessType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<Vec<LabellingEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical_attackers: Option<Vec<ArgRef>>,
}

/// Point query: is `argument_id` accepted under `semantics` and `mode`?
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize)]
pub struct AcceptsResult {
    pub accepted: bool,
    /// `"in"`, `"out"`, `"undec"`, or `"varies"`.
    pub status: String,
    pub unanimous: bool,
    pub witness: WitnessPayload,
}

/// Parse `source`, solve `semantics` extensions, and test acceptance of
/// `argument_id` under credulous or skeptical reasoning.
pub fn accepts(
    source: &str,
    argument_id: usize,
    semantics: Semantics,
    mode: AcceptanceMode,
) -> Result<AcceptsResult, Diagnostic> {
    let doc = parse(source).map_err(|e| Diagnostic {
        message: e.message,
        offset: e.offset,
    })?;
    let model = build_model(&doc);
    let af = dung_framework(&model);
    let arg = ArgumentId(argument_id);

    if !af.arguments.contains(&arg) {
        return Ok(AcceptsResult {
            accepted: false,
            status: "out".to_string(),
            unanimous: true,
            witness: WitnessPayload {
                witness_type: WitnessType::UndefinedArgument,
                labelling: None,
                critical_attackers: None,
            },
        });
    }

    let result = solve(&af, semantics);
    let metadata = analyze_af(&af);
    let labels: Vec<Label> = result
        .labellings
        .iter()
        .map(|lab| lab.get(&arg).copied().unwrap_or(Label::Undec))
        .collect();

    let unanimous = labels.iter().all(|l| *l == labels[0]);
    let credulous_in = labels.contains(&Label::In);
    let skeptical_in = !labels.is_empty() && labels.iter().all(|l| *l == Label::In);
    let accepted = match mode {
        AcceptanceMode::Credulous => credulous_in,
        AcceptanceMode::Skeptical => skeptical_in,
    };

    let status = if unanimous {
        label_str(labels[0]).to_string()
    } else if credulous_in && !skeptical_in {
        "varies".to_string()
    } else {
        dominant_status(&labels)
    };

    let witness = build_witness(&af, &model, &result.labellings, arg, mode, &metadata);

    Ok(AcceptsResult {
        accepted,
        status,
        unanimous,
        witness,
    })
}

/// Parse `source`, project to QBAF, run DF-QuAD, and classify degrees at `threshold`.
pub fn qbaf_evaluate(source: &str, threshold: f64) -> Result<QbafEvaluateResult, Diagnostic> {
    let doc = parse(source).map_err(|e| Diagnostic {
        message: e.message,
        offset: e.offset,
    })?;
    let model = build_model(&doc);
    let qbaf = project_qbaf(&model).map_err(|msg| Diagnostic {
        message: msg,
        offset: 0,
    })?;

    let result = df_quad_run(&qbaf, DEFAULT_MAX_ITERATIONS).map_err(|msg| Diagnostic {
        message: msg,
        offset: 0,
    })?;

    let degrees = qbaf
        .nodes
        .iter()
        .map(|node| {
            let final_degree = result
                .degrees
                .get(&node.id)
                .copied()
                .unwrap_or(node.base_degree);
            QbafDegree {
                id: node.id.0,
                title: title_for(&model, node.id),
                base: node.base_degree,
                final_degree,
                status: classify_degree(final_degree, threshold, !result.converged).to_string(),
            }
        })
        .collect();

    Ok(QbafEvaluateResult {
        semantics: "df_quad".to_string(),
        threshold,
        degrees,
    })
}

fn dominant_status(labels: &[Label]) -> String {
    if labels.iter().all(|l| *l == Label::Out) {
        "out".to_string()
    } else if labels.iter().all(|l| *l == Label::Undec) {
        "undec".to_string()
    } else if labels.contains(&Label::In) {
        "varies".to_string()
    } else {
        "undec".to_string()
    }
}

fn build_witness(
    af: &argdown_model::ArgumentationFramework,
    model: &Model,
    labellings: &[argdown_model::Labeling],
    arg: ArgumentId,
    mode: AcceptanceMode,
    metadata: &AfMetadata,
) -> WitnessPayload {
    let witness_type = classify_witness(labellings, arg, mode, metadata);

    let (labelling_idx, critical_attackers) = match witness_type {
        WitnessType::UndefinedArgument => (None, None),
        WitnessType::AcceptedUncontroversial => (
            labellings
                .iter()
                .position(|lab| lab.get(&arg) == Some(&Label::In)),
            None,
        ),
        WitnessType::AttackedByAccepted => {
            let idx = labellings.iter().position(|lab| {
                lab.get(&arg) == Some(&Label::Out) && in_attackers(af, model, lab, arg).is_some()
            });
            let attackers = idx.and_then(|i| in_attackers(af, model, &labellings[i], arg));
            (idx, attackers)
        }
        WitnessType::UnsupportedCycle => (
            labellings
                .iter()
                .position(|lab| lab.get(&arg) == Some(&Label::Undec)),
            None,
        ),
        WitnessType::MultipleInterpretations => (
            labellings
                .iter()
                .position(|lab| lab.get(&arg) == Some(&Label::In)),
            None,
        ),
        WitnessType::SkepticallyRejected => (
            labellings
                .iter()
                .position(|lab| lab.get(&arg) == Some(&Label::In)),
            None,
        ),
    };

    let labelling = labelling_idx.map(|i| labeling_to_entries(af, model, &labellings[i]));

    WitnessPayload {
        witness_type,
        labelling,
        critical_attackers,
    }
}

fn classify_witness(
    labellings: &[argdown_model::Labeling],
    arg: ArgumentId,
    mode: AcceptanceMode,
    metadata: &AfMetadata,
) -> WitnessType {
    let labels: Vec<Label> = labellings
        .iter()
        .map(|lab| lab.get(&arg).copied().unwrap_or(Label::Undec))
        .collect();
    let credulous_in = labels.contains(&Label::In);
    let skeptical_in = !labels.is_empty() && labels.iter().all(|l| *l == Label::In);

    if labels.iter().all(|l| *l == Label::In) {
        return WitnessType::AcceptedUncontroversial;
    }
    if labels.iter().all(|l| *l == Label::Out) {
        return WitnessType::AttackedByAccepted;
    }
    if labels.iter().all(|l| *l == Label::Undec) && in_cycle(arg, metadata) {
        return WitnessType::UnsupportedCycle;
    }
    if credulous_in && !skeptical_in {
        return match mode {
            AcceptanceMode::Skeptical => WitnessType::MultipleInterpretations,
            AcceptanceMode::Credulous => WitnessType::SkepticallyRejected,
        };
    }
    if labels.contains(&Label::Out) {
        return WitnessType::AttackedByAccepted;
    }
    if labels.contains(&Label::Undec) && in_cycle(arg, metadata) {
        return WitnessType::UnsupportedCycle;
    }
    WitnessType::MultipleInterpretations
}

fn in_cycle(arg: ArgumentId, metadata: &AfMetadata) -> bool {
    metadata
        .strongly_connected_components
        .iter()
        .any(|scc| scc.len() > 1 && scc.contains(&arg))
        || metadata.has_self_attacks && metadata.isolated_arguments.contains(&arg)
}

fn in_attackers(
    af: &argdown_model::ArgumentationFramework,
    model: &Model,
    labeling: &argdown_model::Labeling,
    target: ArgumentId,
) -> Option<Vec<ArgRef>> {
    let attackers: Vec<ArgRef> = af
        .attacks
        .iter()
        .filter(|&&(from, to)| to == target && labeling.get(&from) == Some(&Label::In))
        .map(|&(from, _)| ArgRef {
            id: from.0,
            title: title_for(model, from),
        })
        .collect();
    if attackers.is_empty() {
        None
    } else {
        Some(attackers)
    }
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

    #[test]
    fn accepts_unattacked_skeptical_is_accepted_uncontroversial() {
        let r = accepts("<A>: a", 0, Semantics::Preferred, AcceptanceMode::Skeptical).unwrap();
        assert!(r.accepted);
        assert_eq!(r.status, "in");
        assert!(r.unanimous);
        assert_eq!(r.witness.witness_type, WitnessType::AcceptedUncontroversial);
    }

    #[test]
    fn accepts_attacked_credulous_is_attacked_by_accepted() {
        // A (id 0) attacks B (id 1); B is OUT under preferred.
        let r = accepts(
            "<A>: a\n  -> <B>\n\n<B>: b",
            1,
            Semantics::Preferred,
            AcceptanceMode::Credulous,
        )
        .unwrap();
        assert!(!r.accepted);
        assert_eq!(r.status, "out");
        assert_eq!(r.witness.witness_type, WitnessType::AttackedByAccepted);
        let attackers = r
            .witness
            .critical_attackers
            .expect("critical_attackers present");
        assert_eq!(attackers.len(), 1);
        assert_eq!(attackers[0].id, 0);
    }

    #[test]
    fn accepts_unknown_id_is_undefined_argument() {
        let r = accepts(
            "<A>: a",
            99,
            Semantics::Preferred,
            AcceptanceMode::Credulous,
        )
        .unwrap();
        assert!(!r.accepted);
        assert_eq!(r.witness.witness_type, WitnessType::UndefinedArgument);
    }

    #[test]
    fn accepts_two_cycle_skeptical_is_multiple_interpretations() {
        let src = "<A>: a\n  -> <B>\n\n<B>: b\n  -> <A>";
        let r = accepts(src, 0, Semantics::Preferred, AcceptanceMode::Skeptical).unwrap();
        assert!(!r.accepted);
        assert_eq!(r.status, "varies");
        assert!(!r.unanimous);
        assert_eq!(r.witness.witness_type, WitnessType::MultipleInterpretations);
    }

    #[test]
    fn accepts_two_cycle_credulous_is_skeptically_rejected() {
        let src = "<A>: a\n  -> <B>\n\n<B>: b\n  -> <A>";
        let r = accepts(src, 0, Semantics::Preferred, AcceptanceMode::Credulous).unwrap();
        assert!(r.accepted);
        assert_eq!(r.status, "varies");
        assert_eq!(r.witness.witness_type, WitnessType::SkepticallyRejected);
    }

    #[test]
    fn accepts_two_cycle_grounded_is_unsupported_cycle() {
        let src = "<A>: a\n  -> <B>\n\n<B>: b\n  -> <A>";
        let r = accepts(src, 0, Semantics::Grounded, AcceptanceMode::Skeptical).unwrap();
        assert!(!r.accepted);
        assert_eq!(r.status, "undec");
        assert_eq!(r.witness.witness_type, WitnessType::UnsupportedCycle);
    }

    #[test]
    fn qbaf_evaluate_support_fixture_matches_hand_calculation() {
        let src = "<a>: A {weight: 0.6}\n  + {weight: 0.2} <b>\n\n<b>: B {weight: 0.4}";
        let result = qbaf_evaluate(src, 0.5).unwrap();
        let a = result
            .degrees
            .iter()
            .find(|d| d.title.as_deref() == Some("a"))
            .unwrap();
        assert!((a.final_degree - 0.68).abs() < 1e-6);
        assert_eq!(a.status, "accepted");
    }

    #[test]
    fn qbaf_evaluate_attack_fixture_rejects_below_threshold() {
        let src = "<a>: A {weight: 0.8}\n  - <b>\n\n<b>: B {weight: 0.5}";
        let result = qbaf_evaluate(src, 0.6).unwrap();
        let a = result
            .degrees
            .iter()
            .find(|d| d.title.as_deref() == Some("a"))
            .unwrap();
        assert!((a.final_degree - 0.55).abs() < 1e-6);
        assert_eq!(a.status, "rejected");
    }

    #[test]
    fn qbaf_evaluate_invalid_weight_is_a_diagnostic() {
        let err = qbaf_evaluate("<a>: A {weight: bad}", 0.5).unwrap_err();
        assert_eq!(err.offset, 0);
        assert!(err.message.contains("weight"));
    }
}
