use architect_core::SharedState;
use rmcp::{
    ErrorData,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{CreateMessageRequestParams, ProgressNotificationParam, SamplingMessage},
    service::{RequestContext, RoleServer},
    tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct CallGraphArgs {
    #[schemars(description = "Root directory of the Rust project to analyze")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct RefactorSuggestionArgs {
    #[schemars(
        description = "The function name to get refactoring suggestions for. If omitted, reviews the entire workspace summary."
    )]
    pub function_name: Option<String>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct ImpactAnalysisArgs {
    #[schemars(description = "The function name to analyze the impact of changes")]
    pub function_name: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct ArchLintArgs {
    #[schemars(description = "Root directory of the project to lint")]
    pub path: String,
    #[schemars(
        description = "Optional: Custom architecture rules in JSON format. E.g. {\"forbidden_deps\": [[\"core\", \"web\"]]}"
    )]
    pub rules: Option<String>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct AnalyzeMetricsArgs {
    #[schemars(description = "Root directory of the Rust project to analyze metrics")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct AnalyzeDepsArgs {
    #[schemars(description = "Root directory of the project to analyze dependencies")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct DetectArchPatternArgs {
    #[schemars(description = "Root directory of the project to detect its architectural pattern")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct BlastRadiusArgs {
    #[schemars(description = "Root directory of the project")]
    pub path: String,
    #[schemars(description = "Optional: Target symbol (function name) to analyze")]
    pub target_symbol: Option<String>,
    #[schemars(description = "Optional: Target file path to analyze")]
    pub target_file: Option<String>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct FindDeadCodeArgs {
    #[schemars(description = "Root directory of the project to find dead code")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct SummarizeProjectArgs {
    #[schemars(description = "Root directory of the project to summarize")]
    pub path: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct GenericPathArgs {
    #[schemars(description = "Root directory of the project")]
    pub path: String,
}

pub struct ArchitectTools {
    pub tool_router: ToolRouter<Self>,
    pub state: SharedState,
}

#[tool_router]
impl ArchitectTools {
    #[tool(
        name = "analyze_call_graph",
        description = "Analyzes function calls and definitions across the workspace."
    )]
    pub async fn analyze_call_graph(
        &self,
        Parameters(args): Parameters<CallGraphArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 0.1,
                    total: Some(1.0),
                    message: Some("Starting analysis...".to_string()),
                })
                .await;
        }

        {
            if let Ok(mut last_root) = self.state.last_root.write() {
                *last_root = Some(root.clone());
            }
        }

        let state = self.state.clone();
        let root_for_task = root.clone();

        let (definitions, calls) = tokio::task::spawn_blocking(move || {
            let defs = state.index_definitions(&root_for_task);
            let cls = state.find_calls(&root_for_task, &defs);
            (defs, cls)
        })
        .await
        .map_err(|e| ErrorData::internal_error(format!("Blocking task failed: {}", e), None))?;

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 1.0,
                    total: Some(1.0),
                    message: Some("Analysis complete".to_string()),
                })
                .await;
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

    #[tool(
        name = "analyze_impact",
        description = "Analyzes the impact of changing a specific function by finding all recursive callers."
    )]
    pub async fn analyze_impact(
        &self,
        Parameters(args): Parameters<ImpactAnalysisArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root_opt = self
            .state
            .last_root
            .read()
            .map(|guard| guard.clone())
            .unwrap_or(None);
        let root = root_opt.unwrap_or_else(|| PathBuf::from("."));

        let state = self.state.clone();
        let function_name = args.function_name.clone();

        let result = tokio::task::spawn_blocking(move || {
            state.get_blast_radius(&root, Some(function_name), None)
        })
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "analyze_blast_radius",
        description = "Comprehensive impact analysis across symbols and file dependencies."
    )]
    pub async fn analyze_blast_radius(
        &self,
        Parameters(args): Parameters<BlastRadiusArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();
        let target_symbol = args.target_symbol.clone();
        let target_file = args.target_file.clone();

        let result = tokio::task::spawn_blocking(move || {
            state.get_blast_radius(&root, target_symbol, target_file)
        })
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "find_dead_code",
        description = "Finds functions/symbols that are defined but never used within the workspace."
    )]
    pub async fn find_dead_code(
        &self,
        Parameters(args): Parameters<FindDeadCodeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.find_dead_code(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "summarize_project_structure",
        description = "Provides a high-level summary of the project, including language distribution, entry points, and top-level modules."
    )]
    pub async fn summarize_project_structure(
        &self,
        Parameters(args): Parameters<SummarizeProjectArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.summarize_project_structure(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "scan_security_hotspots",
        description = "Scans for potentially dangerous patterns like eval(), system calls, or hardcoded secrets."
    )]
    pub async fn scan_security_hotspots(
        &self,
        Parameters(args): Parameters<GenericPathArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.scan_security_hotspots(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "extract_api_endpoints",
        description = "Extracts API route definitions and endpoints using framework-specific patterns."
    )]
    pub async fn extract_api_endpoints(
        &self,
        Parameters(args): Parameters<GenericPathArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.extract_api_endpoints(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "analyze_external_coupling",
        description = "Analyzes how deeply third-party libraries penetrate the internal codebase."
    )]
    pub async fn analyze_external_coupling(
        &self,
        Parameters(args): Parameters<GenericPathArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.analyze_external_coupling(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "analyze_test_gap",
        description = "Identifies high-complexity functions that lack corresponding test files."
    )]
    pub async fn analyze_test_gap(
        &self,
        Parameters(args): Parameters<GenericPathArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.analyze_test_gap(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "analyze_outbound_calls",
        description = "Maps all external system interactions (HTTP/gRPC) across the workspace."
    )]
    pub async fn analyze_outbound_calls(
        &self,
        Parameters(args): Parameters<GenericPathArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.analyze_outbound_calls(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "audit_error_handling",
        description = "Audits the codebase for error handling anti-patterns like swallowed exceptions or excessive panics."
    )]
    pub async fn audit_error_handling(
        &self,
        Parameters(args): Parameters<GenericPathArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();

        let result = tokio::task::spawn_blocking(move || state.audit_error_handling(&root))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "lint_architecture",
        description = "Checks for architectural violations like circular dependencies or layer violations across the workspace."
    )]
    pub async fn lint_architecture(
        &self,
        Parameters(args): Parameters<ArchLintArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;
        let state = self.state.clone();
        let rules = args.rules.clone();

        let result = tokio::task::spawn_blocking(move || state.lint_architecture(&root, rules))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "analyze_metrics",
        description = "Calculates cyclomatic complexity and line counts for functions in the workspace."
    )]
    pub async fn analyze_metrics(
        &self,
        Parameters(args): Parameters<AnalyzeMetricsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = self.validate_path(&args.path)?;

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 0.2,
                    total: Some(1.0),
                    message: Some("Calculating metrics...".to_string()),
                })
                .await;
        }

        let state = self.state.clone();
        let root_for_task = root.clone();
        let metrics = tokio::task::spawn_blocking(move || state.get_metrics(&root_for_task))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 1.0,
                    total: Some(1.0),
                    message: Some("Metrics calculation complete".to_string()),
                })
                .await;
        }

        Ok(Json(metrics.to_string()))
    }

    #[tool(
        name = "analyze_dependencies",
        description = "Analyzes imports and dependencies between files across the workspace."
    )]
    pub async fn analyze_dependencies(
        &self,
        Parameters(args): Parameters<AnalyzeDepsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = Path::new(&args.path);

        if !root.exists() {
            return Err(ErrorData::invalid_params(
                format!("Path {} does not exist", args.path),
                None,
            ));
        }

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 0.3,
                    total: Some(1.0),
                    message: Some("Analyzing dependencies...".to_string()),
                })
                .await;
        }

        let deps = self.state.get_dependencies(root);

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 1.0,
                    total: Some(1.0),
                    message: Some("Dependency analysis complete".to_string()),
                })
                .await;
        }

        Ok(Json(deps.to_string()))
    }

    #[tool(
        name = "detect_architecture_pattern",
        description = "Infers the architectural pattern of the project based on folder names."
    )]
    pub async fn detect_architecture_pattern(
        &self,
        Parameters(args): Parameters<DetectArchPatternArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root = Path::new(&args.path);

        if !root.exists() {
            return Err(ErrorData::invalid_params(
                format!("Path {} does not exist", args.path),
                None,
            ));
        }

        if let Some(token) = context.meta.get_progress_token() {
            let _ = context
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: token,
                    progress: 0.1,
                    total: Some(1.0),
                    message: Some("Scanning folders for patterns...".to_string()),
                })
                .await;
        }

        let result = self.state.detect_architecture_pattern(root);

        Ok(Json(result.to_string()))
    }

    #[tool(
        name = "request_refactor_suggestion",
        description = "Requests AI-driven refactoring suggestions based on call graph analysis."
    )]
    pub async fn request_refactor_suggestion(
        &self,
        Parameters(args): Parameters<RefactorSuggestionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<String>, ErrorData> {
        let root_opt = self
            .state
            .last_root
            .read()
            .map(|guard| guard.clone())
            .unwrap_or(None);
        let summary_text = if let Some(root) = root_opt {
            let definitions = self.state.index_definitions(&root);

            // AI 토큰 최적화: 전체 함수 목록 대신, 타겟 함수와 연관된(Blast Radius) 심볼들만 추출
            let context_symbols = if let Some(ref target) = args.function_name {
                let blast = self
                    .state
                    .get_blast_radius(&root, Some(target.clone()), None);
                let related: Vec<String> = blast["symbol_chain"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v["callee_name"].as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                if related.is_empty() {
                    definitions.keys().take(50).cloned().collect::<Vec<_>>() // 최소한의 맥락
                } else {
                    related
                }
            } else {
                definitions.keys().take(100).cloned().collect::<Vec<_>>() // 전체 요약 시 상위 100개로 제한
            };

            format!(
                "Workspace: {}\nTotal Definitions Found: {}\nRelevant/Related Functions: {:?}",
                root.display(),
                definitions.len(),
                context_symbols
            )
        } else {
            "No workspace analyzed yet.".to_string()
        };

        let target = args
            .function_name
            .unwrap_or_else(|| "the whole workspace".to_string());

        let sampling_msg = SamplingMessage::user_text(format!(
            "The following is a code map summary based on the Call Graph of the current project:\n{}\n\nPlease provide refactoring suggestions for '{}' from an architectural perspective. Specifically, suggest ways to improve coupling and cohesion.",
            summary_text, target
        ));

        let sampling_result = context
            .peer
            .create_message(CreateMessageRequestParams::new(vec![sampling_msg], 1000))
            .await
            .map_err(|e| ErrorData::internal_error(format!("Sampling failed: {}", e), None))?;

        let ai_suggestion = sampling_result
            .message
            .content
            .first()
            .and_then(|c| c.as_text())
            .map(|t| t.text.clone())
            .unwrap_or_else(|| "Failed to receive a response from the AI.".to_string());

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

    /// DRY: 경로 유효성 검사 및 PathBuf 반환을 위한 헬퍼
    fn validate_path(&self, path_str: &str) -> Result<PathBuf, ErrorData> {
        let root = Path::new(path_str);
        if !root.exists() {
            return Err(ErrorData::invalid_params(
                format!("Path {} does not exist", path_str),
                None,
            ));
        }
        Ok(root.to_path_buf())
    }
}
