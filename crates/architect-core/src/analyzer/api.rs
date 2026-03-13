use std::path::Path;
use serde_json::{Value, json};
use tree_sitter::{Parser, Query, QueryCursor};
use crate::languages::LanguageProvider;
use std::cell::RefCell;

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new(Parser::new());
}

pub struct ApiAnalyzer;

impl ApiAnalyzer {
    pub fn analyze(&self, path: &Path, content: &str, provider: &dyn LanguageProvider) -> Vec<Value> {
        PARSER.with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
            let lang = provider.language();
            if let Err(_) = parser.set_language(&lang) {
                return Vec::new();
            }

            let tree = match parser.parse(content, None) {
                Some(t) => t,
                None => return Vec::new(),
            };

            let mut results = Vec::new();
            if let Ok(query) = Query::new(&lang, provider.api_query()) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

                for m in matches {
                    for capture in m.captures {
                        let name = &content[capture.node.byte_range()];
                        let start_pos = capture.node.start_position();
                        
                        results.push(json!({
                            "type": "API Endpoint",
                            "route_info": name,
                            "file": path.display().to_string(),
                            "line": start_pos.row + 1
                        }));
                    }
                }
            }
            results
        })
    }
}
