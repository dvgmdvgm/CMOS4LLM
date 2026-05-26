use async_trait::async_trait;
use std::collections::HashMap;

use super::{BackendStatus, ClassifyResult, CompletionRequest, InferenceBackend, InferenceError};

pub struct MockBackend {
    responses: HashMap<String, String>,
    default_response: String,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            default_response: r#"{"category": "unknown", "confidence": 0.5, "reasoning": "mock response"}"#.to_string(),
        }
    }

    pub fn with_response(mut self, prompt_contains: &str, response: &str) -> Self {
        self.responses.insert(prompt_contains.to_string(), response.to_string());
        self
    }

    pub fn with_default(mut self, response: &str) -> Self {
        self.default_response = response.to_string();
        self
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceBackend for MockBackend {
    async fn complete(&self, request: CompletionRequest) -> Result<String, InferenceError> {
        for (key, response) in &self.responses {
            if request.user_prompt.contains(key) {
                return Ok(response.clone());
            }
        }
        Ok(self.default_response.clone())
    }

    async fn classify(&self, _text: &str, categories: &[&str]) -> Result<ClassifyResult, InferenceError> {
        Ok(ClassifyResult {
            category: categories.first().unwrap_or(&"unknown").to_string(),
            confidence: 0.5,
            reasoning: Some("mock classification".to_string()),
        })
    }

    async fn health_check(&self) -> Result<BackendStatus, InferenceError> {
        Ok(BackendStatus::Available { model: "mock".to_string() })
    }
}
