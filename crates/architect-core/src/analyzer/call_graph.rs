use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};
use crate::languages::LanguageProvider;
use architect_types::{FnDefinition, CallInfo};
use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new(Parser::new());
}

pub struct SymbolAnalyzer;

impl SymbolAnalyzer {
    pub fn index_definitions(&self, path: &Path, content: &str, provider: &dyn LanguageProvider) -> Vec<(String, FnDefinition)> {
        PARSER.with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
            let lang = provider.language();
            if let Err(e) = parser.set_language(&lang) {
                tracing::error!("Error loading grammar for {:?}: {}", path, e);
                return Vec::new();
            }

            let tree = match parser.parse(content, None) {
                Some(t) => t,
                None => return Vec::new(),
            };

            let query_str = provider.fn_query();
            let query = match Query::new(&lang, query_str) {
                Ok(q) => q,
                Err(e) => {
                    tracing::error!("Invalid function query for {:?}: {}", path, e);
                    return Vec::new();
                }
            };

            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
            let mut defs = Vec::new();

            for m in matches {
                for capture in m.captures {
                    let name = &content[capture.node.byte_range()];
                    let start_pos = capture.node.start_position();
                    
                    defs.push((name.to_string(), FnDefinition {
                        file: path.to_path_buf(),
                        line: start_pos.row + 1,
                    }));
                }
            }
            defs
        })
    }

    pub fn find_calls(
        &self, 
        path: &Path, 
        content: &str, 
        provider: &dyn LanguageProvider, 
        definitions: &HashMap<String, Vec<FnDefinition>>
    ) -> Vec<CallInfo> {
        PARSER.with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
            let lang = provider.language();
            if let Err(e) = parser.set_language(&lang) {
                tracing::error!("Error loading grammar for {:?}: {}", path, e);
                return Vec::new();
            }

            let query_str = "(call_expression function: (identifier) @call_name) (call_expression (identifier) @call_name)";
            let tree = match parser.parse(content, None) {
                Some(t) => t,
                None => return Vec::new(),
            };
            let mut result = Vec::new();

            if let Ok(query) = Query::new(&lang, query_str) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

                for m in matches {
                    for capture in m.captures {
                        let node = capture.node;
                        let call_name = &content[node.byte_range()];
                        if let Some(defs) = definitions.get(call_name) {
                            let mut caller_name = None;
                            let mut current = node.parent();
                            while let Some(parent) = current {
                                let kind = parent.kind();
                                if kind.contains("function") || kind.contains("method") {
                                    if let Some(name_node) = parent.child_by_field_name("name") {
                                        caller_name = Some(content[name_node.byte_range()].to_string());
                                        break;
                                    }
                                }
                                current = parent.parent();
                            }

                            for def in defs {
                                result.push(CallInfo {
                                    caller_file: path.to_path_buf(),
                                    caller_line: node.start_position().row + 1,
                                    caller_name: caller_name.clone(),
                                    callee_name: call_name.to_string(),
                                    callee_defined_at: format!("{}:{}", def.file.display(), def.line)
                                });
                            }
                        }
                    }
                }
            }
            result
        })
    }
}
