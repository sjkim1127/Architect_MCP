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
            resources: vec![
                RawResource::new("architect://call-graph/summary", "Call Graph Summary")
                    .with_description("Provides a summary of the most recently analyzed call graph")
                    .with_mime_type("application/json")
                    .no_annotation(),
                RawResource::new("architect://visual/mermaid", "Architecture Visual Map (Mermaid)")
                    .with_description("Generates a Mermaid.js diagram of the call graph")
                    .with_mime_type("text/plain")
                    .no_annotation(),
                RawResource::new("architect://metrics/debt", "Technical Debt & Metrics")
                    .with_description("Calculates cyclomatic complexity and identifies 'Hell Functions'")
                    .with_mime_type("application/json")
                    .no_annotation(),
            ],
            ..Default::default()
        })
    }

    pub async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
    ) -> Result<ReadResourceResult, ErrorData> {
        let root_opt = self.state.last_root.lock().unwrap().clone();

        match request.uri.as_str() {
            "architect://call-graph/summary" => {
                let summary = if let Some(root) = root_opt {
                    let definitions = self.state.index_definitions(&root);
                    json!({
                        "workspace": root.display().to_string(),
                        "total_definitions": definitions.len(),
                        "definitions": definitions.keys().collect::<Vec<_>>()
                    }).to_string()
                } else {
                    json!({ "error": "No workspace analyzed. Call 'analyze_call_graph' first." }).to_string()
                };
                self.wrap_text_resource(request.uri, "application/json", summary)
            }
            "architect://visual/mermaid" => {
                let diagram = self.state.get_mermaid_diagram();
                self.wrap_text_resource(request.uri, "text/plain", diagram)
            }
            "architect://metrics/debt" => {
                let metrics = if let Some(root) = root_opt {
                    self.state.get_metrics(&root)
                } else {
                    json!({ "error": "No workspace analyzed. Call 'analyze_call_graph' first." })
                };
                self.wrap_text_resource(request.uri, "application/json", metrics.to_string())
            }
            _ => Err(ErrorData::invalid_params("Unknown resource URI", None)),
        }
    }

    fn wrap_text_resource(&self, uri: String, mime_type: &str, text: String) -> Result<ReadResourceResult, ErrorData> {
        Ok(ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
            uri,
            mime_type: Some(mime_type.to_string()),
            text,
            meta: None,
        }]))
    }
}
