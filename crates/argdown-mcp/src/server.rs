//! The rmcp boundary: adapts pure `argdown_tools` results into MCP tool responses.

use argdown_model::Semantics;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{ErrorData, Json, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use argdown_tools::{
    AcceptanceMode, AcceptsResult, ArgRef, DungResult, ExtensionsResult, Format, InspectAfResult,
    ParseResult, QbafEvaluateResult, ToolError, accepts, extensions, inspect_af, model_export,
    qbaf_evaluate, summarize,
};

/// Inline source input shared by every tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SourceInput {
    /// The Argdown source text to analyze.
    pub source: String,
}

/// Optional serialization format for `export_model`.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Json,
    Yaml,
}

impl From<ExportFormat> for Format {
    fn from(f: ExportFormat) -> Self {
        match f {
            ExportFormat::Json => Format::Json,
            ExportFormat::Yaml => Format::Yaml,
        }
    }
}

/// Source plus optional export format (default JSON).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportModelInput {
    /// The Argdown source text to analyze.
    pub source: String,
    #[serde(default)]
    pub format: ExportFormat,
}

/// Source plus Dung semantics (default preferred).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExtensionsInput {
    /// The Argdown source text to analyze.
    pub source: String,
    #[serde(default = "default_preferred")]
    #[schemars(with = "input_schema::SemanticsSchema")]
    pub semantics: Semantics,
}

/// Credulous vs skeptical acceptance mode for `accepts`.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum AcceptsMode {
    #[default]
    Credulous,
    Skeptical,
}

impl From<AcceptsMode> for AcceptanceMode {
    fn from(mode: AcceptsMode) -> Self {
        match mode {
            AcceptsMode::Credulous => AcceptanceMode::Credulous,
            AcceptsMode::Skeptical => AcceptanceMode::Skeptical,
        }
    }
}

/// Point query: is `argument_id` accepted under `semantics` and `mode`?
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AcceptsInput {
    /// The Argdown source text to analyze.
    pub source: String,
    /// Arena id of the argument to query.
    pub argument_id: usize,
    #[serde(default = "default_preferred")]
    #[schemars(with = "input_schema::SemanticsSchema")]
    pub semantics: Semantics,
    #[serde(default)]
    pub mode: AcceptsMode,
}

fn default_preferred() -> Semantics {
    Semantics::Preferred
}

fn default_threshold() -> f64 {
    0.5
}

/// Source plus optional DF-QuAD acceptance threshold (default 0.5).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct QbafInput {
    /// The Argdown source text to analyze.
    pub source: String,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn invalid_source(d: argdown_tools::Diagnostic) -> ErrorData {
    ErrorData::invalid_params(d.message, Some(json!({ "offset": d.offset })))
}

fn grounded_to_dung(result: ExtensionsResult) -> DungResult {
    let labelling = result.labellings.into_iter().next().unwrap_or_default();
    let mut in_ = Vec::new();
    let mut out = Vec::new();
    let mut undec = Vec::new();
    for entry in labelling {
        let arg = ArgRef {
            id: entry.id,
            title: entry.title,
        };
        match entry.label.as_str() {
            "in" => in_.push(arg),
            "out" => out.push(arg),
            _ => undec.push(arg),
        }
    }
    DungResult { in_, out, undec }
}

mod input_schema {
    use schemars::JsonSchema;

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
}

/// The Argdown MCP server. Stateless.
#[derive(Debug, Clone)]
pub struct ArgdownServer;

#[tool_router]
impl ArgdownServer {
    #[tool(
        name = "parse",
        description = "Parse Argdown source; returns a syntactic summary, or a diagnostic with a byte offset on failure. Prefer inline `source`."
    )]
    fn parse(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Json<ParseResult> {
        Json(summarize(&source))
    }

    #[tool(
        name = "export_model",
        description = "Returns the resolved Layer B model (statements, arguments, PCS roles, dialectical edges, conflicts) as JSON or YAML — not the raw AST or source. Prefer inline `source`; optional `format` (json|yaml, default json)."
    )]
    fn export_model(
        &self,
        Parameters(ExportModelInput { source, format }): Parameters<ExportModelInput>,
    ) -> Result<String, ErrorData> {
        match model_export(&source, format.into()) {
            Ok(text) => Ok(text),
            Err(ToolError::Parse(d)) => Err(invalid_source(d)),
            Err(ToolError::Serialize(msg)) => Err(ErrorData::internal_error(msg, None)),
        }
    }

    #[tool(
        name = "inspect_af",
        description = "Project the Layer B model to a Dung argumentation framework: list arguments, attack edges, and structural metadata (SCCs, acyclicity). Prefer inline `source`."
    )]
    fn inspect_af(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Result<Json<InspectAfResult>, ErrorData> {
        match inspect_af(&source) {
            Ok(result) => Ok(Json(result)),
            Err(d) => Err(invalid_source(d)),
        }
    }

    #[tool(
        name = "extensions",
        description = "Compute Dung-style extensions under the chosen semantics (grounded, preferred, stable, complete). Returns labellings and extension sets. Default semantics: preferred. Prefer inline `source`."
    )]
    fn extensions(
        &self,
        Parameters(ExtensionsInput { source, semantics }): Parameters<ExtensionsInput>,
    ) -> Result<Json<ExtensionsResult>, ErrorData> {
        match extensions(&source, semantics) {
            Ok(result) => Ok(Json(result)),
            Err(d) => Err(invalid_source(d)),
        }
    }

    #[tool(
        name = "accepts",
        description = "Point query: is a specific argument accepted under credulous or skeptical reasoning for the chosen semantics? Returns status and a structured witness. Default semantics: preferred; default mode: credulous. Prefer inline `source`."
    )]
    fn accepts(
        &self,
        Parameters(AcceptsInput {
            source,
            argument_id,
            semantics,
            mode,
        }): Parameters<AcceptsInput>,
    ) -> Result<Json<AcceptsResult>, ErrorData> {
        match accepts(&source, argument_id, semantics, mode.into()) {
            Ok(result) => Ok(Json(result)),
            Err(d) => Err(invalid_source(d)),
        }
    }

    #[tool(
        name = "dung_extensions",
        description = "DEPRECATED: use `extensions` with semantics=\"grounded\". Will be removed in v2. Computes the grounded extension; returns IN/OUT/UNDEC arguments. Prefer inline `source`."
    )]
    fn dung_extensions(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Result<Json<DungResult>, ErrorData> {
        match extensions(&source, Semantics::Grounded) {
            Ok(result) => Ok(Json(grounded_to_dung(result))),
            Err(d) => Err(invalid_source(d)),
        }
    }

    #[tool(
        name = "qbaf_evaluate",
        description = "Compute QBAF DF-QuAD degrees for arguments. Projects argument weights and support/attack edge weights, iterates to fixpoint (max 500 iterations), and classifies each argument as accepted/rejected/undec at the threshold (default 0.5). Prefer inline `source`."
    )]
    fn qbaf_evaluate(
        &self,
        Parameters(QbafInput { source, threshold }): Parameters<QbafInput>,
    ) -> Result<Json<QbafEvaluateResult>, ErrorData> {
        match qbaf_evaluate(&source, threshold) {
            Ok(result) => Ok(Json(result)),
            Err(d) => Err(invalid_source(d)),
        }
    }
}

#[tool_handler(
    instructions = "Argdown argumentation toolchain. Tools: parse (syntactic summary/diagnostics), export_model (Layer B model as JSON or YAML), inspect_af (projected Dung AF), extensions (Dung labellings/extension sets; default semantics preferred), accepts (point query with witness), qbaf_evaluate (QBAF DF-QuAD degrees), dung_extensions (DEPRECATED — use extensions with semantics=grounded). Prefer inline `source`."
)]
impl ServerHandler for ArgdownServer {}
