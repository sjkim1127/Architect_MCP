use architect_types::FnDefinition;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Clone)]
pub struct SharedState {
    pub last_root: Arc<Mutex<Option<PathBuf>>>,
    pub cached_definitions: Arc<Mutex<HashMap<String, Vec<FnDefinition>>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            last_root: Arc::new(Mutex::new(None)),
            cached_definitions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn index_definitions(&self, root: &Path) -> HashMap<String, Vec<FnDefinition>> {
        let mut definitions: HashMap<String, Vec<FnDefinition>> = HashMap::new();
        let mut parser = Parser::new();
        let lang = tree_sitter_rust::language();
        parser.set_language(&lang.into()).expect("Error loading Rust grammar");

        let query_str = "(function_item name: (identifier) @fn_name)";
        let query_lang = tree_sitter_rust::language();
        let query = Query::new(&query_lang.into(), query_str).unwrap();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") && !entry.path().to_string_lossy().contains("/target/") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let tree = parser.parse(&content, None).unwrap();
                    let mut cursor = QueryCursor::new();
                    let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

                    for m in matches {
                        for capture in m.captures {
                            let name = &content[capture.node.byte_range()];
                            let start_pos = capture.node.start_position();
                            
                            definitions.entry(name.to_string()).or_default().push(FnDefinition {
                                file: entry.path().to_path_buf(),
                                line: start_pos.row + 1,
                            });
                        }
                    }
                }
            }
        }
        definitions
    }

    pub fn find_calls(&self, root: &Path, definitions: &HashMap<String, Vec<FnDefinition>>) -> Vec<Value> {
        let mut result = Vec::new();
        let mut parser = Parser::new();
        let lang = tree_sitter_rust::language();
        parser.set_language(&lang.into()).expect("Error loading Rust grammar");

        let query_str = "(call_expression function: (identifier) @call_name)";
        let query_lang = tree_sitter_rust::language();
        let query = Query::new(&query_lang.into(), query_str).unwrap();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") && !entry.path().to_string_lossy().contains("/target/") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let tree = parser.parse(&content, None).unwrap();
                    let mut cursor = QueryCursor::new();
                    let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

                    for m in matches {
                        for capture in m.captures {
                            let call_name = &content[capture.node.byte_range()];
                            if let Some(defs) = definitions.get(call_name) {
                                for def in defs {
                                    result.push(json!({
                                        "caller_file": entry.path().to_string_lossy(),
                                        "caller_line": capture.node.start_position().row + 1,
                                        "callee_name": call_name,
                                        "callee_defined_at": format!("{}:{}", def.file.display(), def.line)
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }
}
