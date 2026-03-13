use rmcp::{
    handler::server::{
        router::prompt::PromptRouter,
        wrapper::Parameters,
    },
    model::{PromptMessage, PromptMessageRole},
    prompt_router, prompt, ErrorData,
};
use architect_core::SharedState;
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
    #[prompt(name = "architect-review", description = "Provides architectural review for a specific function")]
    pub async fn architect_review(&self, Parameters(args): Parameters<ReviewArgs>) -> Result<Vec<PromptMessage>, ErrorData> {
        Ok(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "당신은 Senior Software Architect입니다. 다음 함수 '{}'의 구현과 의존성 관계를 분석하여,\n\
                    구조적 결함(Tight Coupling, Violation of Single Responsibility 등)이 있는지 리뷰해 주세요.\n\
                    필요하다면 `analyze_call_graph` 도구를 사용하여 전체적인 문맥을 파악한 뒤 답변해 주십시오.",
                    args.function_name
                )
            )
        ])
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
