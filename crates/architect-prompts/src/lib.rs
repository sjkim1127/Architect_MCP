use architect_core::SharedState;
use rmcp::{
    ErrorData,
    handler::server::{router::prompt::PromptRouter, wrapper::Parameters},
    model::{PromptMessage, PromptMessageRole},
    prompt, prompt_router,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema, Default)]
pub struct ReviewArgs {
    #[schemars(description = "The function name to review for architectural integrity")]
    pub function_name: String,
}

pub struct ArchitectPrompts {
    pub prompt_router: PromptRouter<Self>,
    pub state: SharedState,
}

#[prompt_router]
impl ArchitectPrompts {
    #[prompt(
        name = "architect-review",
        description = "Provides architectural review for a specific function"
    )]
    pub async fn architect_review(
        &self,
        Parameters(args): Parameters<ReviewArgs>,
    ) -> Result<Vec<PromptMessage>, ErrorData> {
        Ok(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "You are a Senior Software Architect. Review the implementation and dependencies of the function '{}' to identify structural flaws such as Tight Coupling or Violation of Single Responsibility.\n\n\
                    Please utilize the following tools for your analysis:\n\
                    1. `analyze_call_graph`: Understand the overall function call structure\n\
                    2. `analyze_impact`: Identify other functions affected by modifying this function\n\
                    3. `lint_architecture`: Check for structural violations like circular references\n\
                    4. `architect://metrics/debt`: Check complexity and technical debt\n\
                    5. `architect://visual/mermaid`: Visualize the call relationship\n\n\
                    Based on this information, provide specific improvement suggestions.",
                args.function_name
            ),
        )])
    }

    #[prompt(
        name = "architect-refactor-suggestion",
        description = "Analyzes high-complexity or dead code and suggests refactoring strategies"
    )]
    pub async fn refactor_suggestion(
        &self,
        _parameters: Parameters<()>,
    ) -> Result<Vec<PromptMessage>, ErrorData> {
        Ok(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "You are an expert Refactoring Consultant. Please analyze the following resources to identify 'Hell Functions' and 'Dead Code':\n\n\
            1. `architect://metrics/debt`: To find functions with high cyclomatic complexity\n\
            2. `architect://analysis/dead-code`: To find unused symbols that can be safely removed\n\
            3. `architect://analysis/structure`: To understand the project's overall context\n\n\
            Based on these reports, provide a prioritized list of refactoring tasks, explaining the benefits of each."
                .to_string(),
        )])
    }

    #[prompt(
        name = "architect-security-audit",
        description = "Guides the AI through a security-focused audit of the codebase"
    )]
    pub async fn security_audit(
        &self,
        _parameters: Parameters<()>,
    ) -> Result<Vec<PromptMessage>, ErrorData> {
        Ok(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            "You are a Security Auditor. Perform a deep security review of the project by examining these specific resources:\n\n\
            1. `architect://analysis/security`: Review the detected security hotspots\n\
            2. `architect://analysis/api`: Examine API endpoints for potential input validation or authorization issues\n\
            3. `architect://analysis/dead-code`: Check if 'dead' code contains sensitive logic or backdoors\n\n\
            Categorize your findings by severity (Critical, High, Medium, Low) and provide remediation steps for each."
                .to_string(),
        )])
    }
}

impl ArchitectPrompts {
    pub fn new(state: SharedState) -> Self {
        Self {
            prompt_router: Self::prompt_router(),
            state,
        }
    }
}
