use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::SubLmError;
use crate::service::{BackendStatus, ClassifyResult, CompletionRequest, InferenceService, SubLmConfig};

pub struct OllamaRuntime {
    client: Client,
    config: SubLmConfig,
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

impl OllamaRuntime {
    pub fn new(config: SubLmConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build HTTP client");

        Self { client, config }
    }

    pub fn with_defaults() -> Self {
        Self::new(SubLmConfig::default())
    }

    async fn generate(
        &self,
        system: &str,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, SubLmError> {
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature,
                num_predict: max_tokens,
            },
        };

        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
            }

            match self
                .client
                .post(format!("{}/api/generate", self.config.endpoint))
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        last_error = Some(SubLmError::RequestFailed(format!(
                            "HTTP {}: {}",
                            status, body
                        )));
                        continue;
                    }
                    let ollama_resp: OllamaResponse = resp
                        .json()
                        .await
                        .map_err(|e| SubLmError::InvalidResponse(e.to_string()))?;
                    return Ok(ollama_resp.response);
                }
                Err(e) if e.is_timeout() => {
                    last_error = Some(SubLmError::Timeout(self.config.timeout_secs));
                }
                Err(e) if e.is_connect() => {
                    return Err(SubLmError::Unavailable(format!(
                        "cannot connect to Ollama at {}: {}",
                        self.config.endpoint, e
                    )));
                }
                Err(e) => {
                    last_error = Some(SubLmError::RequestFailed(e.to_string()));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| SubLmError::RequestFailed("unknown error".into())))
    }
}

#[async_trait]
impl InferenceService for OllamaRuntime {
    async fn complete(&self, request: CompletionRequest) -> Result<String, SubLmError> {
        self.generate(
            &request.system_prompt,
            &request.user_prompt,
            request.max_tokens,
            request.temperature,
        )
        .await
    }

    async fn classify(
        &self,
        text: &str,
        categories: &[&str],
    ) -> Result<ClassifyResult, SubLmError> {
        let system = "You are a classifier. Respond with ONLY a JSON object: {\"category\": \"<one of the given categories>\", \"confidence\": <0.0-1.0>, \"reasoning\": \"<brief explanation>\"}";
        let prompt = format!(
            "Classify the following text into one of these categories: [{}]\n\nText:\n{}",
            categories.join(", "),
            text
        );

        let response = self.generate(system, &prompt, 200, 0.1).await?;
        parse_classify_response(&response, categories)
    }

    async fn health_check(&self) -> Result<BackendStatus, SubLmError> {
        let resp = self
            .client
            .get(format!("{}/api/tags", self.config.endpoint))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| SubLmError::Unavailable(e.to_string()))?;

        if !resp.status().is_success() {
            return Ok(BackendStatus::Unavailable {
                reason: format!("HTTP {}", resp.status()),
            });
        }

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| SubLmError::InvalidResponse(e.to_string()))?;

        let model_prefix = self
            .config
            .model
            .split(':')
            .next()
            .unwrap_or(&self.config.model);
        let model_available = tags.models.iter().any(|m| m.name.starts_with(model_prefix));

        if model_available {
            Ok(BackendStatus::Available {
                model: self.config.model.clone(),
            })
        } else {
            Ok(BackendStatus::Unavailable {
                reason: format!("model '{}' not found in Ollama", self.config.model),
            })
        }
    }

    async fn summarize(&self, text: &str, max_length: u32) -> Result<String, SubLmError> {
        let system = "You are a summarizer. Produce a concise summary that captures the key points. Output ONLY the summary, no preamble.";
        let prompt = format!(
            "Summarize the following text in at most {} tokens:\n\n{}",
            max_length, text
        );
        self.generate(system, &prompt, max_length, 0.3).await
    }

    async fn extract_json(&self, text: &str, schema_hint: &str) -> Result<String, SubLmError> {
        let system = "You are a structured data extractor. Output ONLY valid JSON matching the requested schema. No markdown, no explanation.";
        let prompt = format!(
            "Extract data from the following text into this JSON schema:\n{}\n\nText:\n{}",
            schema_hint, text
        );
        let response = self.generate(system, &prompt, 2000, 0.1).await?;

        let trimmed = response.trim();
        if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
            return Ok(trimmed[start..=end].to_string());
        }
        if let (Some(start), Some(end)) = (trimmed.find('['), trimmed.rfind(']')) {
            return Ok(trimmed[start..=end].to_string());
        }
        Err(SubLmError::InvalidResponse(
            "no JSON object or array found in response".into(),
        ))
    }
}

fn parse_classify_response(response: &str, categories: &[&str]) -> Result<ClassifyResult, SubLmError> {
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
