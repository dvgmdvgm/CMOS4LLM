use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{BackendStatus, ClassifyResult, CompletionRequest, InferenceBackend, InferenceError};

pub struct OllamaBackend {
    client: Client,
    endpoint: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

impl OllamaBackend {
    pub fn new(endpoint: &str, model: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    async fn generate(&self, system: &str, prompt: &str, max_tokens: u32, temperature: f32) -> Result<String, InferenceError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature,
                num_predict: max_tokens,
            },
        };

        let mut last_error = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
            }

            match self.client
                .post(format!("{}/api/generate", self.endpoint))
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        last_error = Some(InferenceError::RequestFailed(
                            format!("HTTP {}: {}", status, body)
                        ));
                        continue;
                    }
                    let ollama_resp: OllamaResponse = resp.json().await
                        .map_err(|e| InferenceError::InvalidResponse(e.to_string()))?;
                    return Ok(ollama_resp.response);
                }
                Err(e) if e.is_timeout() => {
                    last_error = Some(InferenceError::Timeout(120));
                }
                Err(e) if e.is_connect() => {
                    return Err(InferenceError::Unavailable(
                        format!("cannot connect to Ollama at {}: {}", self.endpoint, e)
                    ));
                }
                Err(e) => {
                    last_error = Some(InferenceError::RequestFailed(e.to_string()));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| InferenceError::RequestFailed("unknown error".into())))
    }
}

#[async_trait]
impl InferenceBackend for OllamaBackend {
    async fn complete(&self, request: CompletionRequest) -> Result<String, InferenceError> {
        self.generate(
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

        let response = self.generate(system, &prompt, 200, 0.1).await?;

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
        let resp = self.client
            .get(format!("{}/api/tags", self.endpoint))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| InferenceError::Unavailable(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(BackendStatus::Unavailable {
                reason: format!("HTTP {}", resp.status()),
            });
        }

        let tags: OllamaTagsResponse = resp.json().await
            .map_err(|e| InferenceError::InvalidResponse(e.to_string()))?;

        let model_available = tags.models.iter().any(|m| m.name.starts_with(&self.model.split(':').next().unwrap_or(&self.model).to_string()));

        if model_available {
            Ok(BackendStatus::Available { model: self.model.clone() })
        } else {
            Ok(BackendStatus::Unavailable {
                reason: format!("model '{}' not found in Ollama", self.model),
            })
        }
    }
}
