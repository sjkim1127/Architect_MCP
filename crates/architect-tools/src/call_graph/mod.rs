use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CallGraphArgs {
    #[schemars(description = "Root directory of the Rust project to analyze")]
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct CallGraphTool {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CallGraphTool {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Analyzes function calls and definitions across the workspace.")]
    fn analyze_call_graph(&self, _params: Parameters<CallGraphArgs>) -> String {
        "Hello from CallGraphTool".to_string()
    }
}

#[tool_handler]
impl ServerHandler for CallGraphTool {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Architect MCP server for codebase analysis".to_string())
    }
}
