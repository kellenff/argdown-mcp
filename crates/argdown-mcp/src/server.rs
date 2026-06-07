//! The rmcp boundary: adapts pure `tools` results into MCP tool responses.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::{ErrorData, Json, ServerHandler, tool, tool_handler, tool_router};
use serde_json::json;

use crate::tools::{self, DungResult, ParseResult, SourceInput, ToolError};

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
        Json(tools::summarize(&source))
    }

    #[tool(
        name = "export_model",
        description = "Returns the resolved Layer B model (statements, arguments, PCS roles, dialectical edges, conflicts) as JSON — not the raw AST or source. Prefer inline `source`."
    )]
    fn export_model(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Result<String, ErrorData> {
        match tools::model_json(&source) {
            Ok(json) => Ok(json),
            Err(ToolError::Parse(d)) => Err(ErrorData::invalid_params(
                d.message,
                Some(json!({ "offset": d.offset })),
            )),
            Err(ToolError::Serialize(msg)) => Err(ErrorData::internal_error(msg, None)),
        }
    }

    #[tool(
        name = "dung_extensions",
        description = "Compute the grounded extension under Dung's abstract argumentation framework; returns IN/OUT/UNDEC arguments. Prefer inline `source`."
    )]
    fn dung_extensions(
        &self,
        Parameters(SourceInput { source }): Parameters<SourceInput>,
    ) -> Result<Json<DungResult>, ErrorData> {
        match tools::dung(&source) {
            Ok(result) => Ok(Json(result)),
            Err(d) => Err(ErrorData::invalid_params(
                d.message,
                Some(json!({ "offset": d.offset })),
            )),
        }
    }
}

#[tool_handler(
    instructions = "Argdown argumentation toolchain. Tools: parse (syntactic summary/diagnostics), export_model (resolved Layer B model as JSON), dung_extensions (grounded IN/OUT/UNDEC). Prefer inline `source`."
)]
impl ServerHandler for ArgdownServer {}
