use rmcp::{
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Parameters, Json},
    },
    model::{ProgressNotificationParam, CreateMessageRequestParams, SamplingMessage},
    service::{RequestContext, RoleServer},
    tool_router, tool, ErrorData,
};
use architect_core::SharedState;
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct CallGraphArgs {
    #[schemars(description = "Root directory of the Rust project to analyze")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct RefactorSuggestionArgs {
    #[schemars(description = "The function name to get refactoring suggestions for. If omitted, reviews the entire workspace summary.")]
    pub function_name: Option<String>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct ImpactAnalysisArgs {
    #[schemars(description = "The function name to analyze the impact of changes")]
    pub function_name: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct ArchLintArgs {
    #[schemars(description = "Optional: Custom architecture rules in JSON format. E.g. {\"forbidden_deps\": [[\"core\", \"web\"]]}")]
    pub rules: Option<String>,
}

pub struct ArchitectTools {
    pub tool_router: ToolRouter<Self>,
    pub state: SharedState,
}

#[tool_router]
impl ArchitectTools {
    #[tool(name = "analyze_call_graph", description = "Analyzes function calls and definitions across the workspace.")]
    pub async fn analyze_call_graph(
        &self,
        Parameters(args): Parameters<CallGraphArgs>,
        context: RequestContext<RoleServer>
    ) -> Result<Json<String>, ErrorData> {
        let root = Path::new(&args.path);

        if !root.exists() {
            return Err(ErrorData::invalid_params(format!("Path {} does not exist", args.path), None));
        }

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context.peer.notify_progress(ProgressNotificationParam {
                progress_token: token,
                progress: 0.1,
                total: Some(1.0),
                message: Some("Starting analysis...".to_string()),
            }).await;
        }

        {
            let mut last_root = self.state.last_root.lock().unwrap();
            *last_root = Some(root.to_path_buf());
        }

        let definitions = self.state.index_definitions(root);
        
        if let Some(token) = context.meta.get_progress_token() {
            let _ = context.peer.notify_progress(ProgressNotificationParam {
                progress_token: token,
                progress: 0.5,
                total: Some(1.0),
                message: Some("Building call graph...".to_string()),
            }).await;
        }

        let calls = self.state.find_calls(root, &definitions);

        {
            let mut cached_defs = self.state.cached_definitions.lock().unwrap();
            *cached_defs = definitions.clone();
            
            let mut cached_calls = self.state.cached_calls.lock().unwrap();
            *cached_calls = calls.clone();
        }

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context.peer.notify_progress(ProgressNotificationParam {
                progress_token: token,
                progress: 1.0,
                total: Some(1.0),
                message: Some("Analysis complete".to_string()),
            }).await;
        }

        let result = json!({
            "status": "success",
            "workspace_root": args.path,
            "total_definitions_found": definitions.len(),
            "total_calls_mapped": calls.len(),
            "calls": calls
        });

        Ok(Json(result.to_string()))
    }

    #[tool(name = "analyze_impact", description = "Analyzes the impact of changing a specific function by finding all recursive callers.")]
    pub async fn analyze_impact(
        &self,
        Parameters(args): Parameters<ImpactAnalysisArgs>,
        _context: RequestContext<RoleServer>
    ) -> Result<Json<String>, ErrorData> {
        let impacted = self.state.get_impact_analysis(&args.function_name);
        
        let result = json!({
            "function": args.function_name,
            "impacted_callers": impacted,
            "total_impacted": impacted.len()
        });

        Ok(Json(result.to_string()))
    }

    #[tool(name = "lint_architecture", description = "Checks for architectural violations like circular dependencies or layer violations.")]
    pub async fn lint_architecture(
        &self,
        Parameters(args): Parameters<ArchLintArgs>,
        _context: RequestContext<RoleServer>
    ) -> Result<Json<String>, ErrorData> {
        let calls = self.state.cached_calls.lock().unwrap();
        let mut violations = Vec::new();

        // 1. Basic Circular Dependency Detection (Direct recursion for now)
        for call in calls.iter() {
            if let Some(ref caller) = call.caller_name {
                if caller == &call.callee_name {
                    violations.push(json!({
                        "type": "Direct recursion",
                        "function": caller,
                        "file": call.caller_file.display().to_string()
                    }));
                }
            }
        }

        // 2. Layer Violation (Example: core should not depend on server)
        // This is a placeholder for the DSL/Rules logic
        if let Some(rules_str) = args.rules {
             // In a real implementation, parse JSON rules and check
             violations.push(json!({"info": "Custom rules parsing not fully implemented", "rules_received": rules_str}));
        }

        let result = json!({
            "status": if violations.is_empty() { "pass" } else { "fail" },
            "violations": violations
        });

        Ok(Json(result.to_string()))
    }

    #[tool(name = "request_refactor_suggestion", description = "Requests AI-driven refactoring suggestions based on call graph analysis.")]
    pub async fn request_refactor_suggestion(
        &self,
        Parameters(args): Parameters<RefactorSuggestionArgs>,
        context: RequestContext<RoleServer>
    ) -> Result<Json<String>, ErrorData> {
        let root_opt = self.state.last_root.lock().unwrap().clone();
        let summary_text = if let Some(root) = root_opt {
             let definitions = self.state.index_definitions(&root);
             format!("Workspace: {}\nTotal Definitions: {}\nFunctions: {:?}", root.display(), definitions.len(), definitions.keys().collect::<Vec<_>>())
        } else {
            "No workspace analyzed yet.".to_string()
        };

        let target = args.function_name.unwrap_or_else(|| "the whole workspace".to_string());
        
        let sampling_msg = SamplingMessage::user_text(format!(
            "다음은 현재 프로젝트의 Call Graph 기반 코드 맵 요약입니다:\n{}\n\n'{}'에 대해 아키텍처 관점에서의 리팩토링 제안을 해주세요. 특히 결합도(Coupling)와 응집도(Cohesion)를 개선할 수 있는 방안을 구체적으로 알려주세요.",
            summary_text, target
        ));

        let sampling_result = context.peer.create_message(CreateMessageRequestParams::new(
            vec![sampling_msg],
            1000
        )).await.map_err(|e| ErrorData::internal_error(format!("Sampling failed: {}", e), None))?;

        let ai_suggestion = sampling_result.message.content.first()
            .and_then(|c| c.as_text())
            .map(|t| t.text.clone())
            .unwrap_or_else(|| "AI로부터 응답을 받지 못했습니다.".to_string());

        Ok(Json(ai_suggestion))
    }
}

impl ArchitectTools {
    pub fn new(state: SharedState) -> Self {
        Self {
            tool_router: Self::tool_router(),
            state,
        }
    }
}
