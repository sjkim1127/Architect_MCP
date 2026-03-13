use architect_core::SharedState;
use architect_tools::ArchitectTools;
use architect_prompts::ArchitectPrompts;
use architect_resources::ArchitectResources;
use rmcp::{
    ServerHandler, ServiceExt, transport::stdio,
    model::{
        ServerCapabilities, ServerInfo, GetPromptResult, ListResourcesResult,
        ReadResourceResult, PaginatedRequestParams, ReadResourceRequestParams,
        CompleteRequestParams, CompleteResult, CompletionInfo, GetPromptRequestParams,
        ListPromptsResult, CallToolRequestParams, CallToolResult, ListToolsResult,
    },
    handler::server::{
        prompt::PromptContext, tool::ToolCallContext,
    },
    service::{RequestContext, RoleServer},
    ErrorData,
};

struct ArchitectServer {
    tools: ArchitectTools,
    prompts: ArchitectPrompts,
    resources: ArchitectResources,
    state: SharedState,
}

impl ArchitectServer {
    fn new() -> Self {
        let state = SharedState::new();
        Self {
            tools: ArchitectTools::new(state.clone()),
            prompts: ArchitectPrompts::new(state.clone()),
            resources: ArchitectResources::new(state.clone()),
            state,
        }
    }
}

impl ServerHandler for ArchitectServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build()
        )
        .with_instructions("Refined Modular Architect MCP server".to_string())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tcc = ToolCallContext::new(&self.tools, request, context);
        self.tools.tool_router.call(tcc).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let items = self.tools.tool_router.list_all();
        Ok(ListToolsResult::with_all_items(items))
    }

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, ErrorData> {
        let mut completions = Vec::new();

        if request.argument.name == "function_name" {
            let value = request.argument.value.to_lowercase();
            let cached = self.state.cached_definitions.lock().unwrap();
            for name in cached.keys() {
                if name.to_lowercase().contains(&value) {
                    completions.push(name.clone());
                }
            }
        }

        let info = CompletionInfo::new(completions)
            .map_err(|e| ErrorData::internal_error(e, None))?;
            
        Ok(CompleteResult::new(info))
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>
    ) -> Result<ListPromptsResult, ErrorData> {
        let items = self.prompts.prompt_router.list_all();
        Ok(ListPromptsResult::with_all_items(items))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<RoleServer>
    ) -> Result<GetPromptResult, ErrorData> {
        let pc = PromptContext::new(&self.prompts, request.name, request.arguments, context);
        self.prompts.prompt_router.get_prompt(pc).await
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        self.resources.list_resources(request).await
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        self.resources.read_resource(request).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Modular Architect MCP server");

    let service = ArchitectServer::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    
    Ok(())
}
