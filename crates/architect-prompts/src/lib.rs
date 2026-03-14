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
}

impl ArchitectPrompts {
    pub fn new(state: SharedState) -> Self {
        Self {
            prompt_router: Self::prompt_router(),
            state,
        }
    }
}
