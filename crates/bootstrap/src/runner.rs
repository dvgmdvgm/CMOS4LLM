use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::config::{InferenceBackendType, ProjectConfig};
use crate::context::PipelineContext;
use crate::graph_store::GraphStore;
use crate::inference::mock::MockBackend;
use crate::inference::ollama::OllamaBackend;
use crate::inference::api::ApiBackend;
use crate::inference::InferenceBackend;
use crate::phases::ast_sweep::AstSweepPhase;
use crate::phases::conventions::ConventionMiningPhase;
use crate::phases::docs::DocsIngestionPhase;
use crate::phases::elicitation::PolicyElicitationPhase;
use crate::phases::git_mining::GitMiningPhase;
use crate::phases::patterns::PatternDetectionPhase;
use crate::phases::rejected::RejectedApproachesPhase;
use crate::phases::schema::SchemaExtractionPhase;
use crate::phases::{Phase, PhaseError};
use crate::progress::ProgressReporter;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("config error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("graph error: {0}")]
    Graph(#[from] crate::graph_store::GraphError),
    #[error("phase {0} failed: {1}")]
    PhaseFailed(String, String),
    #[error("no inference backend available")]
    NoInferenceBackend,
}

pub struct PipelineRunner {
    pub project_name: String,
    pub root_path: PathBuf,
    pub resume: bool,
    pub no_interactive: bool,
    pub skip_phases: Vec<u8>,
    pub backend_override: Option<String>,
    pub model_override: Option<String>,
}

impl PipelineRunner {
    pub fn new(project_name: &str, root_path: PathBuf) -> Self {
        Self {
            project_name: project_name.to_string(),
            root_path,
            resume: false,
            no_interactive: false,
            skip_phases: vec![],
            backend_override: None,
            model_override: None,
        }
    }

    pub fn run(&self) -> Result<(), RunnerError> {
        let config_path = self.root_path.join(".cmos").join("config.toml");
        let mut config = if config_path.exists() {
            ProjectConfig::load(&config_path)?
        } else {
            let cfg = ProjectConfig::default_for(&self.project_name);
            cfg.save(&config_path).ok();
            cfg
        };

        if let Some(ref model) = self.model_override {
            config.inference.model = model.clone();
        }

        if self.no_interactive {
            config.bootstrap.interactive = false;
        }

        for phase in &self.skip_phases {
            if !config.bootstrap.skip_phases.contains(phase) {
                config.bootstrap.skip_phases.push(*phase);
            }
        }

        let db_path = self.root_path.join(".cmos").join("graph.db");
        let graph = GraphStore::open(&db_path)?;
        graph.ensure_project(
            &self.project_name,
            &config.project.name,
            &self.root_path.to_string_lossy(),
        )?;

        let inference = self.create_inference_backend(&config);
        let progress = ProgressReporter::new(8);

        let mut ctx = PipelineContext {
            config,
            root_path: self.root_path.clone(),
            graph,
            inference,
            progress,
        };

        let last_completed = if self.resume {
            ctx.graph.get_last_completed_phase(&self.project_name)?
        } else {
            None
        };

        let phases: Vec<Box<dyn Phase>> = vec![
            Box::new(AstSweepPhase),
            Box::new(SchemaExtractionPhase),
            Box::new(PatternDetectionPhase),
            Box::new(ConventionMiningPhase),
            Box::new(GitMiningPhase),
            Box::new(RejectedApproachesPhase),
            Box::new(DocsIngestionPhase),
            Box::new(PolicyElicitationPhase),
        ];

        for phase in &phases {
            let phase_id = phase.id().as_u8();

            if let Some(last) = last_completed
                && phase_id <= last
            {
                continue;
            }

            if ctx.config.bootstrap.skip_phases.contains(&phase_id) {
                ctx.progress.start_phase(phase_id, phase.id().name());
                ctx.progress.phase_skipped("configured to skip");
                continue;
            }

            ctx.progress.start_phase(phase_id, phase.id().name());
            let started_at = chrono::Utc::now().to_rfc3339();
            let start = Instant::now();

            match phase.run(&mut ctx) {
                Ok(output) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    ctx.progress.phase_done(duration_ms);

                    let stats = serde_json::json!({
                        "nodes_created": output.nodes_created,
                        "edges_created": output.edges_created,
                        "duration_ms": duration_ms,
                        "warnings": output.warnings.len(),
                    });

                    ctx.graph.save_checkpoint(
                        &self.project_name,
                        phase_id,
                        "completed",
                        &started_at,
                        Some(&chrono::Utc::now().to_rfc3339()),
                        Some(&stats.to_string()),
                    )?;

                    for warning in &output.warnings {
                        ctx.progress.phase_warning(warning);
                    }
                }
                Err(PhaseError::Skipped(reason)) => {
                    ctx.progress.phase_skipped(&reason);
                    ctx.graph.save_checkpoint(
                        &self.project_name,
                        phase_id,
                        "skipped",
                        &started_at,
                        Some(&chrono::Utc::now().to_rfc3339()),
                        Some(&format!("{{\"reason\":\"{}\"}}", reason)),
                    )?;
                }
                Err(e) => {
                    ctx.progress.phase_warning(&format!("FAILED: {}", e));
                    ctx.graph.save_checkpoint(
                        &self.project_name,
                        phase_id,
                        "failed",
                        &started_at,
                        Some(&chrono::Utc::now().to_rfc3339()),
                        Some(&format!("{{\"error\":\"{}\"}}", e)),
                    )?;

                    if phase.id().requires_inference() {
                        continue;
                    }

                    return Err(RunnerError::PhaseFailed(
                        phase.id().name().to_string(),
                        e.to_string(),
                    ));
                }
            }
        }

        let node_counts = ctx.graph.count_nodes_by_kind(&self.project_name)?;
        let edge_counts = ctx.graph.count_edges_by_kind(&self.project_name)?;
        let total_nodes: i64 = node_counts.iter().map(|(_, c)| c).sum();
        let total_edges: i64 = edge_counts.iter().map(|(_, c)| c).sum();
        ctx.progress.summary(total_nodes as usize, total_edges as usize);

        Ok(())
    }

    fn create_inference_backend(&self, config: &ProjectConfig) -> Arc<dyn InferenceBackend> {
        let backend_type = if let Some(ref override_str) = self.backend_override {
            match override_str.as_str() {
                "api" => InferenceBackendType::Api,
                _ => InferenceBackendType::Ollama,
            }
        } else {
            config.inference.backend.clone()
        };

        match backend_type {
            InferenceBackendType::Ollama => {
                Arc::new(OllamaBackend::new(&config.inference.endpoint, &config.inference.model))
            }
            InferenceBackendType::Api => {
                match ApiBackend::from_env("https://api.anthropic.com", &config.inference.model) {
                    Ok(backend) => Arc::new(backend),
                    Err(_) => {
                        eprintln!("WARNING: API backend unavailable (CMOS_API_KEY not set), using mock");
                        Arc::new(MockBackend::new())
                    }
                }
            }
        }
    }
}
