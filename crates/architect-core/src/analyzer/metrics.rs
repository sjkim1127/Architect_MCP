use std::path::Path;
use serde_json::{Value, json};
use tree_sitter::{Parser, Node};
use crate::languages::LanguageProvider;
use std::cell::RefCell;

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new(Parser::new());
}

pub struct MetricsAnalyzer;

impl MetricsAnalyzer {
    pub fn analyze(&self, path: &Path, content: &str, provider: &dyn LanguageProvider) -> Vec<Value> {
        let results = PARSER.with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
            if let Err(e) = parser.set_language(&provider.language()) {
                tracing::error!("Error loading grammar for {:?}: {}", path, e);
                return Vec::new();
            }
            
            let tree = match parser.parse(content, None) {
                Some(t) => t,
                None => return Vec::new(),
            };
            let mut cursor = tree.walk();
            let mut results = Vec::new();
            
            self.traverse_for_metrics(&mut cursor, content, path, &mut results, provider);
            results
        });
        results
    }

    fn traverse_for_metrics<'a>(
        &self, 
        cursor: &mut tree_sitter::TreeCursor<'a>, 
        content: &str, 
        path: &Path, 
        results: &mut Vec<Value>, 
        provider: &dyn LanguageProvider
    ) {
        let node = cursor.node();
        let kind = node.kind();
        
        if kind.contains("function") || kind.contains("method") {
            let name = node.child_by_field_name("name")
                .map(|n| &content[n.byte_range()])
                .unwrap_or("unknown");
            
            let complexity = self.calculate_complexity(node, provider);
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            let length = end_line - start_line + 1;

            results.push(json!({
                "function": name,
                "file": path.display().to_string(),
                "line": start_line,
                "cyclomatic_complexity": complexity,
                "length": length,
                "is_hell_function": complexity > 10 || length > 100
            }));
        }

        if cursor.goto_first_child() {
            loop {
                self.traverse_for_metrics(cursor, content, path, results, provider);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    fn calculate_complexity(&self, node: Node, provider: &dyn LanguageProvider) -> usize {
        let mut complexity = 1;
        let mut cursor = node.walk();
        self.count_complexity_nodes(&mut cursor, &mut complexity, provider);
        complexity
    }

    fn count_complexity_nodes(&self, cursor: &mut tree_sitter::TreeCursor, count: &mut usize, provider: &dyn LanguageProvider) {
        let node = cursor.node();
        if provider.is_complexity_node(node.kind()) {
            *count += 1;
        }

        if cursor.goto_first_child() {
            loop {
                self.count_complexity_nodes(cursor, count, provider);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }
}
