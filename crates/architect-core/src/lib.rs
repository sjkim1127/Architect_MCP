use architect_types::{FnDefinition, CallInfo};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use ignore::WalkBuilder;
use rayon::prelude::*;

pub mod languages;
pub mod analyzer;

use languages::{LanguageRegistry, LanguageProvider};
use analyzer::{MetricsAnalyzer, DependencyAnalyzer, SymbolAnalyzer};

#[derive(Default, Clone, Debug)]
pub struct WorkspaceState {
    pub cached_definitions: HashMap<String, Vec<FnDefinition>>,
    pub cached_calls: Vec<CallInfo>,
}

#[derive(Clone)]
pub struct SharedState {
    pub last_root: Arc<Mutex<Option<PathBuf>>>,
    pub workspace_cache: Arc<Mutex<HashMap<PathBuf, WorkspaceState>>>,
    pub registry: Arc<LanguageRegistry>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            last_root: Arc::new(Mutex::new(None)),
            workspace_cache: Arc::new(Mutex::new(HashMap::new())),
            registry: Arc::new(LanguageRegistry::new()),
        }
    }

    fn get_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in WalkBuilder::new(root)
            .hidden(true)
            .git_ignore(true)
            .build()
            .filter_map(|e| e.ok()) {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                files.push(entry.path().to_path_buf());
            }
        }
        files
    }

    /// DRY: 병렬 파일 처리를 위한 공통 헬퍼 메서드
    fn process_files_parallel<F, R>(&self, root: &Path, f: F) -> Vec<R> 
    where 
        F: Fn(&Path, &str, &dyn LanguageProvider) -> R + Sync + Send,
        R: Send
    {
        let files = self.get_files(root);
        files.par_iter().filter_map(|path| {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if let Some(provider) = self.registry.get_provider(ext) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    return Some(f(path, &content, provider.as_ref()));
                }
            }
            None
        }).collect()
    }

    pub fn index_definitions(&self, root: &Path) -> HashMap<String, Vec<FnDefinition>> {
        let analyzer = SymbolAnalyzer;
        let all_defs = self.process_files_parallel(root, |path, content, provider| {
            analyzer.index_definitions(path, content, provider)
        });

        let mut definitions: HashMap<String, Vec<FnDefinition>> = HashMap::new();
        for file_defs in all_defs {
            for (name, def) in file_defs {
                definitions.entry(name).or_default().push(def);
            }
        }

        // 워크스페이스별 캐시에 저장
        if let Ok(mut cache) = self.workspace_cache.lock() {
            cache.entry(root.to_path_buf()).or_default().cached_definitions = definitions.clone();
        }

        definitions
    }

    pub fn find_calls(&self, root: &Path, definitions: &HashMap<String, Vec<FnDefinition>>) -> Vec<CallInfo> {
        let analyzer = SymbolAnalyzer;
        let all_calls = self.process_files_parallel(root, |path, content, provider| {
            analyzer.find_calls(path, content, provider, definitions)
        });

        let calls: Vec<CallInfo> = all_calls.into_iter().flatten().collect();

        // 워크스페이스별 캐시에 저장
        if let Ok(mut cache) = self.workspace_cache.lock() {
            cache.entry(root.to_path_buf()).or_default().cached_calls = calls.clone();
        }

        calls
    }

    pub fn get_metrics(&self, root: &Path) -> Value {
        let analyzer = MetricsAnalyzer;
        let all_metrics = self.process_files_parallel(root, |path, content, provider| {
            analyzer.analyze(path, content, provider)
        });

        json!(all_metrics.into_iter().flatten().collect::<Vec<_>>())
    }

    pub fn get_dependencies(&self, root: &Path) -> Value {
        let analyzer = DependencyAnalyzer;
        
        let files = self.get_files(root);
        let all_deps: HashMap<String, Vec<String>> = files.par_iter().filter_map(|path| {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if let Some(provider) = self.registry.get_provider(ext) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let imports = analyzer.analyze(path, &content, provider.as_ref());
                    if !imports.is_empty() {
                        let rel_path = path.strip_prefix(root).unwrap_or(path).display().to_string();
                        return Some((rel_path, imports));
                    }
                }
            }
            None
        }).collect();

        serde_json::to_value(all_deps).unwrap_or(Value::Null)
    }

    pub fn detect_architecture_pattern(&self, root: &Path) -> Value {
        let mut folder_names = HashSet::new();
        
        for entry in WalkBuilder::new(root)
            .max_depth(Some(4))
            .hidden(true)
            .git_ignore(true)
            .build()
            .filter_map(|e| e.ok()) {
            if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    folder_names.insert(name.to_lowercase());
                }
            }
        }

        let mut patterns = Vec::new();

        let layered_keywords = ["controllers", "services", "repositories", "models", "dto", "api", "views"];
        let layered_score = layered_keywords.iter().filter(|&&k| folder_names.contains(k)).count();
        if layered_score >= 2 {
            patterns.push(json!({ "name": "Layered Architecture", "confidence": layered_score as f32 / layered_keywords.len() as f32 }));
        }

        let hexagonal_keywords = ["domain", "ports", "adapters", "application", "infrastructure"];
        let hexagonal_score = hexagonal_keywords.iter().filter(|&&k| folder_names.contains(k)).count();
        if hexagonal_score >= 2 {
            patterns.push(json!({ "name": "Hexagonal Architecture", "confidence": hexagonal_score as f32 / hexagonal_keywords.len() as f32 }));
        }

        let clean_keywords = ["entities", "use_cases", "interfaces", "frameworks", "drivers"];
        let clean_score = clean_keywords.iter().filter(|&&k| folder_names.contains(k)).count();
        if clean_score >= 2 {
            patterns.push(json!({ "name": "Clean Architecture", "confidence": clean_score as f32 / clean_keywords.len() as f32 }));
        }

        let frontend_keywords = ["components", "containers", "actions", "reducers", "store", "hooks", "pages"];
        let frontend_score = frontend_keywords.iter().filter(|&&k| folder_names.contains(k)).count();
        if frontend_score >= 2 {
            patterns.push(json!({ "name": "Frontend Standard (React/Redux)", "confidence": frontend_score as f32 / frontend_keywords.len() as f32 }));
        }

        json!({
            "detected_patterns": patterns,
            "all_scanned_folders": folder_names.into_iter().collect::<Vec<_>>()
        })
    }

    pub fn get_mermaid_diagram(&self, root: &Path) -> String {
        let cache = self.workspace_cache.lock().unwrap();
        let calls = cache.get(root).map(|s| &s.cached_calls).cloned().unwrap_or_default();
        
        let mut diagram = String::from("graph TD;\n");
        let mut seen = HashSet::new();

        for call in calls.iter() {
            if let Some(ref caller) = call.caller_name {
                let edge = format!("    {} --> {};\n", caller, call.callee_name);
                if !seen.contains(&edge) {
                    diagram.push_str(&edge);
                    seen.insert(edge);
                }
            }
        }
        diagram
    }

    pub fn get_blast_radius(&self, root: &Path, target_symbol: Option<String>, target_file: Option<String>) -> Value {
        let cache = self.workspace_cache.lock().unwrap();
        let calls = cache.get(root).map(|s| &s.cached_calls).cloned().unwrap_or_default();
        
        let mut impacted_symbols = Vec::new();
        let mut impacted_files = HashSet::new();

        // 1. Symbol-level Impact (Call Graph)
        if let Some(symbol) = target_symbol {
            let mut to_visit: Vec<String> = vec![symbol];
            let mut visited = HashSet::new();

            while let Some(callee) = to_visit.pop() {
                if visited.contains(&callee) { continue; }
                visited.insert(callee.clone());

                for call in calls.iter() {
                    if call.callee_name == callee {
                        impacted_symbols.push(call.clone());
                        impacted_files.insert(call.caller_file.display().to_string());
                        if let Some(ref caller) = call.caller_name {
                            to_visit.push(caller.clone());
                        }
                    }
                }
            }
        }

        // 2. File-level Impact (Dependency Graph)
        let deps_value = self.get_dependencies(root);
        if let Some(target_path) = target_file {
            if let Some(deps_map) = deps_value.as_object() {
                let mut to_visit = vec![target_path];
                let mut visited = HashSet::new();

                while let Some(current_file) = to_visit.pop() {
                    if visited.contains(&current_file) { continue; }
                    visited.insert(current_file.clone());

                    for (file, imports) in deps_map {
                        for import in imports.as_array().unwrap_or(&vec![]) {
                            let import_str = import.as_str().unwrap_or("");
                            // 간단한 경로 매칭 (실제 구현에서는 모듈 해석 로직 필요)
                            if import_str.contains(&current_file) {
                                impacted_files.insert(file.clone());
                                to_visit.push(file.clone());
                            }
                        }
                    }
                }
            }
        }

        json!({
            "impacted_symbols_count": impacted_symbols.len(),
            "impacted_files_count": impacted_files.len(),
            "impacted_files": impacted_files.into_iter().collect::<Vec<_>>(),
            "symbol_chain": impacted_symbols
        })
    }

    pub fn find_dead_code(&self, root: &Path) -> Value {
        // 1. 모든 정의(Definitions) 수집
        let definitions = self.index_definitions(root);
        
        // 2. 모든 호출(Calls) 수집
        let calls = self.find_calls(root, &definitions);
        
        // 3. 호출된 대상 이름들 수집
        let called_names: HashSet<String> = calls.iter().map(|c| c.callee_name.clone()).collect();
        
        let mut dead_code = Vec::new();

        // 4. 정의되었으나 호출되지 않은 것 필터링
        for (name, defs) in definitions {
            // main이나 test 등 특수 목적 함수는 제외 (간단한 필터링)
            if name == "main" || name.contains("test") {
                continue;
            }

            if !called_names.contains(&name) {
                for def in defs {
                    dead_code.push(json!({
                        "symbol": name,
                        "file": def.file.display().to_string(),
                        "line": def.line
                    }));
                }
            }
        }

        json!({
            "dead_symbols_count": dead_code.len(),
            "dead_symbols": dead_code
        })
    }

    pub fn lint_architecture(&self, root: &Path, rules_json: Option<String>) -> Value {
        let mut violations = Vec::new();
        let cache = self.workspace_cache.lock().unwrap();
        let calls = cache.get(root).map(|s| &s.cached_calls).cloned().unwrap_or_default();
        let deps_value = self.get_dependencies(root);

        // 1. 규칙 파싱 (예: {"forbidden_deps": [["core", "server"]]})
        let mut forbidden_deps = Vec::new();
        if let Some(json_str) = rules_json {
            if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
                if let Some(forbidden) = v.get("forbidden_deps").and_then(|f| f.as_array()) {
                    for rule in forbidden {
                        if let Some(pair) = rule.as_array() {
                            if pair.len() == 2 {
                                if let (Some(from), Some(to)) = (pair[0].as_str(), pair[1].as_str()) {
                                    forbidden_deps.push((from.to_string(), to.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. 파일 의존성 레이어 위반 체크
        if let Some(deps_map) = deps_value.as_object() {
            for (file, imports) in deps_map {
                for (from_layer, to_layer) in &forbidden_deps {
                    if file.contains(from_layer) {
                        for import in imports.as_array().unwrap_or(&vec![]) {
                            let import_str = import.as_str().unwrap_or("");
                            if import_str.contains(to_layer) {
                                violations.push(json!({
                                    "type": "Layer Violation (Dependency)",
                                    "rule": format!("{} -> {}", from_layer, to_layer),
                                    "file": file,
                                    "violation": format!("Imports '{}'", import_str)
                                }));
                            }
                        }
                    }
                }
            }
        }

        // 3. 호출 그래프 기반 레이어 위반 체크
        for call in calls.iter() {
            let caller_file = call.caller_file.display().to_string();
            for (from_layer, to_layer) in &forbidden_deps {
                if caller_file.contains(from_layer) && call.callee_defined_at.contains(to_layer) {
                    violations.push(json!({
                        "type": "Layer Violation (Call)",
                        "rule": format!("{} -> {}", from_layer, to_layer),
                        "file": caller_file,
                        "function": call.caller_name,
                        "violation": format!("Calls '{}' defined in '{}'", call.callee_name, call.callee_defined_at)
                    }));
                }
            }
        }

        // 4. 순환 참조 체크 (단순 직접 순환 우선)
        for call in calls.iter() {
            if let Some(ref caller) = call.caller_name {
                if caller == &call.callee_name {
                    violations.push(json!({
                        "type": "Circular Dependency (Direct)",
                        "symbol": caller,
                        "file": call.caller_file.display().to_string()
                    }));
                }
            }
        }

        json!({
            "status": if violations.is_empty() { "pass" } else { "fail" },
            "violations_count": violations.len(),
            "violations": violations
        })
    }

    pub fn summarize_project_structure(&self, root: &Path) -> Value {
        let files = self.get_files(root);
        let mut lang_counts = HashMap::new();
        let mut entry_points = Vec::new();
        let mut total_files = 0;

        for path in &files {
            total_files += 1;
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("unknown");
            *lang_counts.entry(ext.to_string()).or_insert(0) += 1;

            // 잠재적 진입점 탐색 (main.rs, app.py, index.js 등)
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if file_name.starts_with("main.") || file_name.starts_with("app.") || file_name.starts_with("index.") || file_name == "lib.rs" {
                entry_points.push(path.strip_prefix(root).unwrap_or(path).display().to_string());
            }
        }

        let mut lang_distribution = Vec::new();
        for (lang, count) in lang_counts {
            let percentage = (count as f32 / total_files as f32) * 100.0;
            lang_distribution.push(json!({
                "language": lang,
                "file_count": count,
                "percentage": format!("{:.1}%", percentage)
            }));
        }

        // 언어 비중순으로 정렬
        lang_distribution.sort_by(|a, b| {
            let a_val = a["file_count"].as_u64().unwrap_or(0);
            let b_val = b["file_count"].as_u64().unwrap_or(0);
            b_val.cmp(&a_val)
        });

        // 상위 수준 모듈(폴더) 목록
        let mut top_modules = HashSet::new();
        for path in &files {
            if let Some(parent) = path.strip_prefix(root).ok().and_then(|p| p.components().next()) {
                let name = parent.as_os_str().to_string_lossy().to_string();
                if name != "src" && !name.starts_with('.') {
                     top_modules.insert(name);
                }
            }
        }

        json!({
            "total_files": total_files,
            "language_distribution": lang_distribution,
            "potential_entry_points": entry_points,
            "top_level_modules": top_modules.into_iter().collect::<Vec<_>>(),
            "detected_architecture": self.detect_architecture_pattern(root)
        })
    }
}
