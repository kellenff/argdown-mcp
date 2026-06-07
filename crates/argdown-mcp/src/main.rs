//! Argdown MCP server: serves `parse` / `export_model` / `dung_extensions`
//! over stdio.

mod server;
mod tools;

use rmcp::ServiceExt;
use rmcp::transport::io::stdio;

use server::ArgdownServer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = ArgdownServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
