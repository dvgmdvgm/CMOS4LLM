use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::RetrievalError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub endpoint: String,
    pub model: String,
    pub timeout_secs: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "nomic-embed-text:latest".to_string(),
            timeout_secs: 30,
        }
    }
}

pub struct EmbeddingClient {
    client: Client,
    config: EmbeddingConfig,
}

#[derive(Serialize)]
struct OllamaEmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

impl EmbeddingClient {
    pub fn new(config: EmbeddingConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build HTTP client");
        Self { client, config }
    }

    pub fn dimension(&self) -> usize {
        if self.config.model.starts_with("nomic-embed-text") {
            768
        } else if self.config.model.starts_with("mxbai-embed-large") {
            1024
        } else {
            768
        }
    }

    pub async fn embed_single(&self, text: &str) -> Result<Vec<f32>, RetrievalError> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| RetrievalError::Embedding("empty response from embedding API".into()))
    }

    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, RetrievalError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = OllamaEmbedRequest {
            model: self.config.model.clone(),
            input: texts.to_vec(),
        };

        let resp = self
            .client
            .post(format!("{}/api/embed", self.config.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    RetrievalError::Embedding(format!(
                        "cannot connect to Ollama at {}: {}",
                        self.config.endpoint, e
                    ))
                } else if e.is_timeout() {
                    RetrievalError::Embedding("embedding request timed out".into())
                } else {
                    RetrievalError::Embedding(e.to_string())
                }
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(RetrievalError::Embedding(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let embed_resp: OllamaEmbedResponse = resp
            .json()
            .await
            .map_err(|e| RetrievalError::Embedding(format!("invalid response: {}", e)))?;

        Ok(embed_resp.embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.model, "nomic-embed-text:latest");
        assert_eq!(config.endpoint, "http://localhost:11434");
    }

    #[test]
    fn test_dimension_detection() {
        let client = EmbeddingClient::new(EmbeddingConfig::default());
        assert_eq!(client.dimension(), 768);

        let client = EmbeddingClient::new(EmbeddingConfig {
            model: "mxbai-embed-large:latest".into(),
            ..Default::default()
        });
        assert_eq!(client.dimension(), 1024);
    }
}
