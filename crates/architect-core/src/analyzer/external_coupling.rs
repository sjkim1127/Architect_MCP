use crate::analyzer::DependencyAnalyzer;
use crate::languages::LanguageProvider;
use serde_json::{Value, json};
use std::path::Path;

pub struct ExternalCouplingAnalyzer;

impl ExternalCouplingAnalyzer {
    pub fn analyze(&self, path: &Path, content: &str, provider: &dyn LanguageProvider) -> Value {
        let dep_analyzer = DependencyAnalyzer;
        let imports = dep_analyzer.analyze(path, content, provider);

        let mut internal_imports = 0;
        let mut external_imports = 0;
        let mut external_list = Vec::new();

        for import in &imports {
            // 프로젝트 내부 경로인지 외부 라이브러리인지 판단 (간단한 로직)
            if import.contains("./") || import.contains("../") || import.contains("crate::") {
                internal_imports += 1;
            } else {
                external_imports += 1;
                external_list.push(import.clone());
            }
        }

        let total = internal_imports + external_imports;
        let penetration = if total > 0 {
            (external_imports as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        json!({
            "file": path.display().to_string(),
            "internal_imports": internal_imports,
            "external_imports": external_imports,
            "penetration_percentage": format!("{:.1}%", penetration),
            "external_libraries": external_list
        })
    }
}
