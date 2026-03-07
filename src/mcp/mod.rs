mod audit;
mod error;
mod local_tools;
pub mod proxy;
mod sandbox;
mod server;

use rmcp::ServiceExt;

/// Run the MCP stdio server.
pub async fn run(
    api_url: String,
    api_key: Option<String>,
    workspace: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = server::StdioMcpServer::new(api_url, api_key, workspace);
    let transport = rmcp::transport::io::stdio();

    let service = server.serve(transport).await?;
    tracing::info!("MCP server running on stdio");
    service.waiting().await?;

    Ok(())
}
