use walkdir::WalkDir;

use crate::context::PipelineContext;
use crate::graph_store::Node;
use crate::inference::CompletionRequest;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct RejectedApproachesPhase;

impl Phase for RejectedApproachesPhase {
    fn id(&self) -> PhaseId {
        PhaseId::RejectedApproaches
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let mut nodes_created = 0;
        let mut warnings = Vec::new();

        let mut markers: Vec<TechDebtMarker> = Vec::new();

        let py_files: Vec<_> = WalkDir::new(&ctx.root_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "__pycache__"
                    && name != "venv"
                    && name != ".venv"
                    && name != "migrations"
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "py"))
            .collect();

        for entry in &py_files {
            let path = entry.path();
            let rel_path = path.strip_prefix(&ctx.root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("HACK") || trimmed.contains("XXX") {
                    markers.push(TechDebtMarker {
                        file_path: rel_path.clone(),
                        line: line_num as u32 + 1,
                        text: trimmed.to_string(),
                    });
                }
            }
        }

        ctx.progress.phase_detail(&format!("TODO/FIXME/HACK found: {}", markers.len()));

        for marker in &markers {
            ctx.graph.insert_node(&Node {
                id: None,
                project_id: ctx.config.project.name.clone(),
                kind: "tech_debt_marker".to_string(),
                label: truncate(&marker.text, 100),
                file_path: Some(marker.file_path.clone()),
                line_start: Some(marker.line),
                line_end: Some(marker.line),
                properties_json: serde_json::json!({
                    "full_text": marker.text,
                    "marker_type": classify_marker(&marker.text),
                }).to_string(),
                phase_id: PhaseId::RejectedApproaches.as_u8(),
            })?;
            nodes_created += 1;
        }

        if !markers.is_empty() && markers.len() <= 50 {
            let batch_text = markers.iter()
                .take(20)
                .map(|m| format!("{}:{} - {}", m.file_path, m.line, m.text))
                .collect::<Vec<_>>()
                .join("\n");

            let system = "You are a code analyst. Analyze these TODO/FIXME/HACK comments and identify any that indicate rejected approaches or architectural decisions. Output JSON array: [{\"index\": N, \"category\": \"rejected_approach|tech_debt|planned_feature|workaround\", \"reasoning\": \"brief explanation\"}]";

            let rt = tokio::runtime::Handle::try_current();
            let response = match rt {
                Ok(handle) => {
                    tokio::task::block_in_place(|| {
                        handle.block_on(ctx.inference.complete(CompletionRequest {
                            system_prompt: system.to_string(),
                            user_prompt: batch_text,
                            max_tokens: 1000,
                            temperature: 0.2,
                        }))
                    })
                }
                Err(_) => {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| PhaseError::Io(std::io::Error::other(e)))?;
                    rt.block_on(ctx.inference.complete(CompletionRequest {
                        system_prompt: system.to_string(),
                        user_prompt: batch_text,
                        max_tokens: 1000,
                        temperature: 0.2,
                    }))
                }
            };

            if let Err(e) = response {
                warnings.push(format!("LM classification failed: {}", e));
            }
        }

        Ok(PhaseOutput {
            nodes_created,
            edges_created: 0,
            warnings,
        })
    }
}

struct TechDebtMarker {
    file_path: String,
    line: u32,
    text: String,
}

fn classify_marker(text: &str) -> &'static str {
    let upper = text.to_uppercase();
    if upper.contains("HACK") || upper.contains("XXX") {
        "workaround"
    } else if upper.contains("FIXME") {
        "bug"
    } else {
        "todo"
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
