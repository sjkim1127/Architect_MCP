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
                    구조적 결함(Tight Coupling, Violation of Single Responsibility 등)이 있는지 리뷰해 주세요.\n\n\
                    분석을 위해 다음 도구들을 적절히 활용해 주십시오:\n\
                    1. `analyze_call_graph`: 전체적인 함수 호출 구조 파악\n\
                    2. `analyze_impact`: 해당 함수 수정 시 영향을 받는 다른 함수들 확인\n\
                    3. `lint_architecture`: 순환 참조 등 구조적 위반 사항 체크\n\
                    4. `architect://metrics/debt`: 복잡도 및 기술 부채 확인\n\
                    5. `architect://visual/mermaid`: 호출 관계 시각화\n\n\
                    위 정보를 바탕으로 구체적인 개선안을 제시해 주십시오.",
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
