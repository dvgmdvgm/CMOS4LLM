use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectSection,
    pub inference: InferenceSection,
    pub bootstrap: BootstrapSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    #[serde(default)]
    pub root: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceSection {
    pub backend: InferenceBackendType,
    pub model: String,
    pub endpoint: String,
    #[serde(default)]
    pub api_fallback: Option<ApiFallbackSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InferenceBackendType {
    Ollama,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiFallbackSection {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapSection {
    #[serde(default = "default_true")]
    pub interactive: bool,
    #[serde(default)]
    pub skip_phases: Vec<u8>,
}

fn default_true() -> bool {
    true
}

impl ProjectConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadFailed(path.to_path_buf(), e))?;
        toml::from_str(&content).map_err(ConfigError::ParseFailed)
    }

    pub fn default_for(project_name: &str) -> Self {
        Self {
            project: ProjectSection {
                name: project_name.to_string(),
                root: None,
            },
            inference: InferenceSection {
                backend: InferenceBackendType::Ollama,
                model: "gemma2:latest".to_string(),
                endpoint: "http://localhost:11434".to_string(),
                api_fallback: Some(ApiFallbackSection {
                    enabled: true,
                    provider: "anthropic".to_string(),
                    model: "claude-haiku-4-5-20251001".to_string(),
                    endpoint: None,
                }),
            },
            bootstrap: BootstrapSection {
                interactive: true,
                skip_phases: vec![],
            },
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::WriteFailed(path.to_path_buf(), e))?;
        }
        let content = toml::to_string_pretty(self).map_err(ConfigError::SerializeFailed)?;
        std::fs::write(path, content)
            .map_err(|e| ConfigError::WriteFailed(path.to_path_buf(), e))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config at {0}: {1}")]
    ReadFailed(PathBuf, std::io::Error),
    #[error("failed to parse config: {0}")]
    ParseFailed(#[from] toml::de::Error),
    #[error("failed to write config at {0}: {1}")]
    WriteFailed(PathBuf, std::io::Error),
    #[error("failed to serialize config: {0}")]
    SerializeFailed(toml::ser::Error),
}
