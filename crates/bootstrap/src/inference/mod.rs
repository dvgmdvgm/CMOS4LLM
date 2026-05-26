pub mod api;
pub mod mock;
pub mod ollama;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("request failed: {0}")]
    RequestFailed(String),
    #[error("timeout after {0}s")]
    Timeout(u64),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
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
pub trait InferenceBackend: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<String, InferenceError>;
    async fn classify(&self, text: &str, categories: &[&str]) -> Result<ClassifyResult, InferenceError>;
    async fn health_check(&self) -> Result<BackendStatus, InferenceError>;
}
