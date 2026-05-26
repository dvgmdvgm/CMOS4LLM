use std::path::Path;
use walkdir::WalkDir;

use crate::context::PipelineContext;
use crate::extractors::django::DjangoExtractor;
use crate::extractors::python::PythonExtractor;
use crate::graph_store::Node;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct AstSweepPhase;

impl Phase for AstSweepPhase {
    fn id(&self) -> PhaseId {
        PhaseId::AstSweep
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let mut extractor = PythonExtractor::new()?;
        let django = DjangoExtractor::new();
        let mut nodes_created = 0;
        let mut warnings = Vec::new();

        let py_files: Vec<_> = WalkDir::new(&ctx.root_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "__pycache__"
                    && name != ".git"
                    && name != "venv"
                    && name != ".venv"
                    && name != "env"
                    && name != "migrations"
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "py")
            })
            .collect();

        let total_files = py_files.len();
        ctx.progress.phase_detail(&format!("{} Python files found", total_files));

        for entry in &py_files {
            let path = entry.path();
            let rel_path = path.strip_prefix(&ctx.root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let source = match std::fs::read(path) {
                Ok(s) => s,
                Err(e) => {
                    warnings.push(format!("cannot read {}: {}", rel_path, e));
                    continue;
                }
            };

            let raw_nodes = match extractor.parse_file(&source, Path::new(&rel_path)) {
                Ok(nodes) => nodes,
                Err(e) => {
                    warnings.push(format!("parse error in {}: {}", rel_path, e));
                    continue;
                }
            };

            let mut batch = Vec::new();
            for raw in &raw_nodes {
                let classified = django.classify_node(raw);
                let node = classified.as_ref().unwrap_or(raw);

                batch.push(Node {
                    id: None,
                    project_id: ctx.config.project.name.clone(),
                    kind: node.kind.clone(),
                    label: node.label.clone(),
                    file_path: Some(node.file_path.clone()),
                    line_start: Some(node.line_start),
                    line_end: Some(node.line_end),
                    properties_json: node.properties.to_string(),
                    phase_id: PhaseId::AstSweep.as_u8(),
                });
            }

            if !batch.is_empty() {
                ctx.graph.insert_nodes_batch(&batch)?;
                nodes_created += batch.len();
            }
        }

        let node_counts = ctx.graph.count_nodes_by_kind(&ctx.config.project.name)?;
        for (kind, count) in &node_counts {
            ctx.progress.phase_detail(&format!("{}: {}", kind, count));
        }

        Ok(PhaseOutput {
            nodes_created,
            edges_created: 0,
            warnings,
        })
    }
}
