use walkdir::WalkDir;

use crate::context::PipelineContext;
use crate::extractors::django::DjangoExtractor;
use crate::graph_store::Node;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct PatternDetectionPhase;

impl Phase for PatternDetectionPhase {
    fn id(&self) -> PhaseId {
        PhaseId::PatternDetection
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let django = DjangoExtractor::new();
        let edges_created = 0;
        let mut nodes_created = 0;
        let mut warnings = Vec::new();

        // Extract URL patterns
        let url_files: Vec<_> = WalkDir::new(&ctx.root_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "node_modules" && name != "__pycache__"
                    && name != "venv" && name != ".venv" && name != "migrations"
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                name == "urls.py" || name == "api_urls.py"
            })
            .collect();

        for entry in &url_files {
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

            let urls = match django.extract_url_patterns(&source, std::path::Path::new(&rel_path)) {
                Ok(u) => u,
                Err(e) => {
                    warnings.push(format!("url extraction error in {}: {}", rel_path, e));
                    continue;
                }
            };

            let batch: Vec<_> = urls.iter().map(|u| {
                crate::graph_store::Node {
                    id: None,
                    project_id: ctx.config.project.name.clone(),
                    kind: u.kind.clone(),
                    label: u.label.clone(),
                    file_path: Some(u.file_path.clone()),
                    line_start: Some(u.line_start),
                    line_end: Some(u.line_end),
                    properties_json: u.properties.to_string(),
                    phase_id: PhaseId::PatternDetection.as_u8(),
                }
            }).collect();

            if !batch.is_empty() {
                ctx.graph.insert_nodes_batch(&batch)?;
                nodes_created += batch.len();
            }
        }

        // Extract middleware chain from settings
        let settings_files: Vec<_> = WalkDir::new(&ctx.root_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "node_modules" && name != "__pycache__"
                    && name != "venv" && name != ".venv"
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                name == "settings.py" || name == "base.py" || name == "production.py"
            })
            .filter(|e| {
                let path_str = e.path().to_string_lossy();
                path_str.contains("settings") || path_str.contains("config")
            })
            .collect();

        for entry in &settings_files {
            let source = match std::fs::read(entry.path()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let middleware = django.extract_settings_middleware(&source);
            if !middleware.is_empty() {
                let rel_path = entry.path().strip_prefix(&ctx.root_path)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .replace('\\', "/");

                for mw in &middleware {
                    let node = Node {
                        id: None,
                        project_id: ctx.config.project.name.clone(),
                        kind: "middleware".to_string(),
                        label: mw.clone(),
                        file_path: Some(rel_path.clone()),
                        line_start: None,
                        line_end: None,
                        properties_json: serde_json::json!({
                            "source": "settings",
                        }).to_string(),
                        phase_id: PhaseId::PatternDetection.as_u8(),
                    };
                    ctx.graph.insert_node(&node)?;
                    nodes_created += 1;
                }

                ctx.progress.phase_detail(&format!("middleware chain: {} layers", middleware.len()));
            }
        }

        let _all_functions = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "function")?;
        let _all_views = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "django_view")?;
        let _all_imports = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "import")?;

        ctx.progress.phase_detail(&format!("urls: {}", nodes_created));
        ctx.progress.phase_detail(&format!("edges: {}", edges_created));

        Ok(PhaseOutput {
            nodes_created,
            edges_created,
            warnings,
        })
    }
}
