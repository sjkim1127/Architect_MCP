use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDefinition {
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfo {
    pub caller_file: PathBuf,
    pub caller_line: usize,
    pub caller_name: Option<String>,
    pub callee_name: String,
    pub callee_defined_at: String,
}
