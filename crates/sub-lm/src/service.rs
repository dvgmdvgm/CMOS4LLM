use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::SubLmError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubLmConfig {
    pub endpoint: String,
    pub model: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for SubLmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "gemma4:latest".to_string(),
            timeout_secs: 120,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub struct ClassifyResult {
    pub category: String,
    pub confidence: f32,
    pub reasoning: Option<String>,
}

#[derive(Debug)]
pub enum BackendStatus {
    Available { model: String },
    Unavailable { reason: String },
}

#[async_trait]
pub trait InferenceService: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<String, SubLmError>;
    async fn classify(&self, text: &str, categories: &[&str]) -> Result<ClassifyResult, SubLmError>;
    async fn health_check(&self) -> Result<BackendStatus, SubLmError>;
    async fn summarize(&self, text: &str, max_length: u32) -> Result<String, SubLmError>;
    async fn extract_json(&self, text: &str, schema_hint: &str) -> Result<String, SubLmError>;
}
