use std::path::Path;
use serde_json::{Value, json};
use tree_sitter::{Parser, Query, QueryCursor};
use crate::languages::LanguageProvider;

pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn analyze(&self, path: &Path, content: &str, provider: &dyn LanguageProvider) -> Vec<Value> {
        let mut parser = Parser::new();
        let lang = provider.language();
        if let Err(_) = parser.set_language(&lang) {
            return Vec::new();
        }

        let tree = match parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        if let Ok(query) = Query::new(&lang, provider.security_query()) {
            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

            for m in matches {
                for capture in m.captures {
                    let name = &content[capture.node.byte_range()];
                    let start_pos = capture.node.start_position();
                    
                    results.push(json!({
                        "issue": "Security Hotspot",
                        "symbol": name,
                        "file": path.display().to_string(),
                        "line": start_pos.row + 1,
                        "description": format!("Potentially dangerous symbol/pattern found: {}", name)
                    }));
                }
            }
        }
        results
    }
}
