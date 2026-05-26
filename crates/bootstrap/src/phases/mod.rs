pub mod ast_sweep;
pub mod conventions;
pub mod docs;
pub mod elicitation;
pub mod git_mining;
pub mod patterns;
pub mod rejected;
pub mod schema;

use crate::context::PipelineContext;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhaseId {
    AstSweep = 1,
    SchemaExtraction = 2,
    PatternDetection = 3,
    ConventionMining = 4,
    GitMining = 5,
    RejectedApproaches = 6,
    DocsIngestion = 7,
    PolicyElicitation = 8,
}

impl PhaseId {
    pub fn all() -> &'static [PhaseId] {
        &[
            PhaseId::AstSweep,
            PhaseId::SchemaExtraction,
            PhaseId::PatternDetection,
            PhaseId::ConventionMining,
            PhaseId::GitMining,
            PhaseId::RejectedApproaches,
            PhaseId::DocsIngestion,
            PhaseId::PolicyElicitation,
        ]
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            PhaseId::AstSweep => "Static AST Sweep",
            PhaseId::SchemaExtraction => "Schema & Domain Extraction",
            PhaseId::PatternDetection => "Architectural Pattern Detection",
            PhaseId::ConventionMining => "Convention Mining",
            PhaseId::GitMining => "Git History Mining",
            PhaseId::RejectedApproaches => "Rejected Approaches Detection",
            PhaseId::DocsIngestion => "Documentation Ingestion",
            PhaseId::PolicyElicitation => "Policy Elicitation",
        }
    }

    pub fn requires_inference(self) -> bool {
        matches!(self, PhaseId::ConventionMining | PhaseId::RejectedApproaches | PhaseId::DocsIngestion)
    }

    pub fn is_interactive(self) -> bool {
        matches!(self, PhaseId::PolicyElicitation)
    }
}

#[derive(Debug)]
pub struct PhaseOutput {
    pub nodes_created: usize,
    pub edges_created: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum PhaseError {
    #[error("extractor error: {0}")]
    Extractor(#[from] crate::extractors::ExtractorError),
    #[error("graph error: {0}")]
    Graph(#[from] crate::graph_store::GraphError),
    #[error("inference error: {0}")]
    Inference(#[from] crate::inference::InferenceError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("phase skipped: {0}")]
    Skipped(String),
}

pub trait Phase {
    fn id(&self) -> PhaseId;

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError>;
}
