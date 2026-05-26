use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{BackendStatus, ClassifyResult, CompletionRequest, InferenceBackend, InferenceError};

pub struct ApiBackend {
    client: Client,
    endpoint: String,
    model: String,
    api_key: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

impl ApiBackend {
    pub fn new(endpoint: &str, model: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn from_env(endpoint: &str, model: &str) -> Result<Self, InferenceError> {
        let api_key = std::env::var("CMOS_API_KEY")
            .map_err(|_| InferenceError::Unavailable("CMOS_API_KEY env var not set".into()))?;
        Ok(Self::new(endpoint, model, &api_key))
    }

    async fn chat(&self, system: &str, user: &str, max_tokens: u32, temperature: f32) -> Result<String, InferenceError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(), content: user.into() },
            ],
            max_tokens,
            temperature,
        };

        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    InferenceError::Timeout(60)
                } else if e.is_connect() {
                    InferenceError::Unavailable(format!("cannot connect to API at {}", self.endpoint))
                } else {
                    InferenceError::RequestFailed(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(InferenceError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        let chat_resp: ChatResponse = resp.json().await
            .map_err(|e| InferenceError::InvalidResponse(e.to_string()))?;

        chat_resp.choices.into_iter().next()
            .map(|c| c.message.content)
            .ok_or_else(|| InferenceError::InvalidResponse("empty choices".into()))
    }
}

#[async_trait]
impl InferenceBackend for ApiBackend {
    async fn complete(&self, request: CompletionRequest) -> Result<String, InferenceError> {
        self.chat(
            &request.system_prompt,
            &request.user_prompt,
            request.max_tokens,
            request.temperature,
        ).await
    }

    async fn classify(&self, text: &str, categories: &[&str]) -> Result<ClassifyResult, InferenceError> {
        let system = "You are a classifier. Respond with ONLY a JSON object: {\"category\": \"<one of the given categories>\", \"confidence\": <0.0-1.0>, \"reasoning\": \"<brief explanation>\"}";
        let prompt = format!(
            "Classify the following text into one of these categories: [{}]\n\nText:\n{}",
            categories.join(", "),
            text
        );

        let response = self.chat(system, &prompt, 200, 0.1).await?;
        let trimmed = response.trim();
        let json_str = if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                &trimmed[start..=end]
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        #[derive(Deserialize)]
        struct ClassifyResponse {
            category: String,
            #[serde(default)]
            confidence: f32,
            #[serde(default)]
            reasoning: Option<String>,
        }

        match serde_json::from_str::<ClassifyResponse>(json_str) {
            Ok(parsed) => Ok(ClassifyResult {
                category: parsed.category,
                confidence: parsed.confidence,
                reasoning: parsed.reasoning,
            }),
            Err(_) => Ok(ClassifyResult {
                category: categories.first().unwrap_or(&"unknown").to_string(),
                confidence: 0.0,
                reasoning: Some(format!("failed to parse response: {}", trimmed)),
            }),
        }
    }

    async fn health_check(&self) -> Result<BackendStatus, InferenceError> {
        Ok(BackendStatus::Available { model: self.model.clone() })
    }
}
