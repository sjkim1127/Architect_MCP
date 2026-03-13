use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FnDefinition {
    pub file: PathBuf,
    pub line: usize,
}
