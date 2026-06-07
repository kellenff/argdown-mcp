//! Argdown MCP server: serves parse, export_model, inspect_af, extensions,
//! accepts, qbaf_evaluate, and the deprecated dung_extensions alias over stdio.

use argdown_mcp::server::ArgdownServer;
use rmcp::ServiceExt;
use rmcp::transport::io::stdio;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ArgdownServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
