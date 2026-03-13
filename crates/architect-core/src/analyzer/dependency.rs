use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};
use crate::languages::LanguageProvider;

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn analyze(&self, _path: &Path, content: &str, provider: &dyn LanguageProvider) -> Vec<String> {
        let mut parser = Parser::new();
        let lang = provider.language();
        parser.set_language(&lang).expect("Error loading grammar");

        let tree = parser.parse(content, None).unwrap();
        if let Ok(query) = Query::new(&lang, provider.import_query()) {
            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
            let mut imports = Vec::new();

            for m in matches {
                for capture in m.captures {
                    let import_text = &content[capture.node.byte_range()];
                    imports.push(import_text.trim().to_string());
                }
            }
            imports
        } else {
            Vec::new()
        }
    }
}
