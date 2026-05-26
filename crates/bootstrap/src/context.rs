use crate::config::ProjectConfig;
use crate::graph_store::GraphStore;
use crate::inference::InferenceBackend;
use crate::progress::ProgressReporter;
use cmos_memory::EventStore;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PipelineContext {
    pub config: ProjectConfig,
    pub root_path: PathBuf,
    pub graph: GraphStore,
    pub inference: Arc<dyn InferenceBackend>,
    pub progress: ProgressReporter,
    pub event_store: Option<EventStore>,
}
