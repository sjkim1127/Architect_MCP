use rmcp::{
    model::{
        ListResourcesResult, ReadResourceResult, RawResource, ResourceContents,
        PaginatedRequestParams, ReadResourceRequestParams, AnnotateAble,
    },
    ErrorData,
};
use architect_core::SharedState;
use serde_json::json;

pub struct ArchitectResources {
    pub state: SharedState,
}

impl ArchitectResources {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: vec![RawResource::new("architect://call-graph/summary", "Call Graph Summary")
                .with_description("Provides a summary of the most recently analyzed call graph")
                .with_mime_type("application/json")
                .no_annotation()],
            ..Default::default()
        })
    }

    pub async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
    ) -> Result<ReadResourceResult, ErrorData> {
        if request.uri == "architect://call-graph/summary" {
            let root_opt = self.state.last_root.lock().unwrap().clone();
            let summary = if let Some(root) = root_opt {
                let definitions = self.state.index_definitions(&root);
                json!({
                    "workspace": root.display().to_string(),
                    "total_definitions": definitions.len(),
                    "definitions": definitions.keys().collect::<Vec<_>>()
                }).to_string()
            } else {
                json!({
                    "error": "No workspace has been analyzed yet. Please call 'analyze_call_graph' tool first."
                }).to_string()
            };

            Ok(ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
                uri: request.uri,
                mime_type: Some("application/json".to_string()),
                text: summary,
                meta: None,
            }]))
        } else {
            Err(ErrorData::invalid_params("Unknown resource URI", None))
        }
    }
}
