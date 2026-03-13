use architect_types::{FnDefinition, CallInfo};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use tree_sitter::{Parser, Query, QueryCursor, Node};

#[derive(Clone)]
pub struct SharedState {
    pub last_root: Arc<Mutex<Option<PathBuf>>>,
    pub cached_definitions: Arc<Mutex<HashMap<String, Vec<FnDefinition>>>>,
    pub cached_calls: Arc<Mutex<Vec<CallInfo>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            last_root: Arc::new(Mutex::new(None)),
            cached_definitions: Arc::new(Mutex::new(HashMap::new())),
            cached_calls: Arc::new(Mutex::new(Vec::new())),
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

    pub fn find_calls(&self, root: &Path, definitions: &HashMap<String, Vec<FnDefinition>>) -> Vec<CallInfo> {
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
                            let node = capture.node;
                            let call_name = &content[node.byte_range()];
                            if let Some(defs) = definitions.get(call_name) {
                                // Find enclosing function name
                                let mut caller_name = None;
                                let mut current = node.parent();
                                while let Some(parent) = current {
                                    if parent.kind() == "function_item" {
                                        if let Some(name_node) = parent.child_by_field_name("name") {
                                            caller_name = Some(content[name_node.byte_range()].to_string());
                                            break;
                                        }
                                    }
                                    current = parent.parent();
                                }

                                for def in defs {
                                    result.push(CallInfo {
                                        caller_file: entry.path().to_path_buf(),
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
            }
        }
        result
    }

    pub fn get_mermaid_diagram(&self) -> String {
        let calls = self.cached_calls.lock().unwrap();
        let mut diagram = String::from("graph TD;\n");
        let mut seen = HashSet::new();

        for call in calls.iter() {
            if let Some(ref caller) = call.caller_name {
                let edge = format!("    {} --> {};\n", caller, call.callee_name);
                if !seen.contains(&edge) {
                    diagram.push_str(&edge);
                    seen.insert(edge);
                }
            } else {
                let edge = format!("    main --> {};\n", call.callee_name);
                 if !seen.contains(&edge) {
                    diagram.push_str(&edge);
                    seen.insert(edge);
                }
            }
        }
        diagram
    }

    pub fn get_impact_analysis(&self, function_name: &str) -> Vec<CallInfo> {
        let calls = self.cached_calls.lock().unwrap();
        let mut impacted = Vec::new();
        let mut to_visit = vec![function_name.to_string()];
        let mut visited = HashSet::new();

        while let Some(callee) = to_visit.pop() {
            if visited.contains(&callee) { continue; }
            visited.insert(callee.clone());

            for call in calls.iter() {
                if call.callee_name == callee {
                    impacted.push(call.clone());
                    if let Some(ref caller) = call.caller_name {
                        to_visit.push(caller.clone());
                    }
                }
            }
        }
        impacted
    }

    pub fn get_metrics(&self, root: &Path) -> Value {
        let mut parser = Parser::new();
        let lang = tree_sitter_rust::language();
        parser.set_language(&lang.into()).expect("Error loading Rust grammar");

        let mut metrics = Vec::new();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "rs") && !entry.path().to_string_lossy().contains("/target/") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let tree = parser.parse(&content, None).unwrap();
                    let mut cursor = tree.walk();
                    
                    self.traverse_for_metrics(&mut cursor, &content, entry.path(), &mut metrics);
                }
            }
        }

        serde_json::to_value(metrics).unwrap_or(Value::Null)
    }

    fn traverse_for_metrics<'a>(&self, cursor: &mut tree_sitter::TreeCursor<'a>, content: &str, path: &Path, results: &mut Vec<Value>) {
        let node = cursor.node();
        if node.kind() == "function_item" {
            let name = node.child_by_field_name("name")
                .map(|n| &content[n.byte_range()])
                .unwrap_or("unknown");
            
            let complexity = self.calculate_complexity(node, content);
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            let length = end_line - start_line + 1;

            results.push(serde_json::json!({
                "function": name,
                "file": path.display().to_string(),
                "line": start_line,
                "cyclomatic_complexity": complexity,
                "length": length,
                "is_hell_function": complexity > 10 || length > 50
            }));
        }

        if cursor.goto_first_child() {
            loop {
                self.traverse_for_metrics(cursor, content, path, results);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }

    fn calculate_complexity(&self, node: Node, _content: &str) -> usize {
        let mut complexity = 1;
        let mut cursor = node.walk();
        self.count_complexity_nodes(&mut cursor, &mut complexity);
        complexity
    }

    fn count_complexity_nodes(&self, cursor: &mut tree_sitter::TreeCursor, count: &mut usize) {
        let node = cursor.node();
        match node.kind() {
            "if_expression" | "for_expression" | "while_expression" | "match_arm" | "&&" | "||" => {
                *count += 1;
            }
            _ => {}
        }

        if cursor.goto_first_child() {
            loop {
                self.count_complexity_nodes(cursor, count);
                if !cursor.goto_next_sibling() { break; }
            }
            cursor.goto_parent();
        }
    }
}
