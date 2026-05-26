pub mod django;
pub mod python;

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractorError {
    #[error("parse error in {0}: {1}")]
    ParseError(String, String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct RawNode {
    pub kind: String,
    pub label: String,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RawEdge {
    pub source_label: String,
    pub source_kind: String,
    pub target_label: String,
    pub target_kind: String,
    pub kind: String,
    pub properties: serde_json::Value,
}

pub trait LanguageExtractor: Send + Sync {
    fn language(&self) -> &str;
    fn file_extensions(&self) -> &[&str];
    fn extract_symbols(&self, source: &[u8], path: &Path) -> Result<Vec<RawNode>, ExtractorError>;
}
