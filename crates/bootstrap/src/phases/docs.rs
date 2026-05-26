use walkdir::WalkDir;

use crate::context::PipelineContext;
use crate::graph_store::Node;
use crate::inference::CompletionRequest;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct DocsIngestionPhase;

impl Phase for DocsIngestionPhase {
    fn id(&self) -> PhaseId {
        PhaseId::DocsIngestion
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let mut nodes_created = 0;
        let mut warnings = Vec::new();

        let md_files: Vec<_> = WalkDir::new(&ctx.root_path)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "node_modules" && name != "venv"
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                let ext = e.path().extension().map(|x| x.to_string_lossy().to_string());
                matches!(ext.as_deref(), Some("md") | Some("rst") | Some("txt"))
                    || name.to_uppercase().starts_with("README")
                    || name.to_uppercase().starts_with("CHANGELOG")
            })
            .collect();

        ctx.progress.phase_detail(&format!("documentation files: {}", md_files.len()));

        for entry in &md_files {
            let path = entry.path();
            let rel_path = path.strip_prefix(&ctx.root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if content.len() < 50 {
                continue;
            }

            let truncated = if content.len() > 4000 {
                let mut end = 4000;
                while !content.is_char_boundary(end) {
                    end -= 1;
                }
                &content[..end]
            } else {
                &content
            };

            let system = "You are a documentation analyzer. Extract key facts from this documentation. Output a JSON array: [{\"kind\": \"doc_fact|domain_term|constraint|architectural_decision\", \"label\": \"short name\", \"description\": \"what it means\"}]. Focus on architectural decisions, constraints, and domain-specific terminology.";

            let rt = tokio::runtime::Handle::try_current();
            let response = match rt {
                Ok(handle) => {
                    tokio::task::block_in_place(|| {
                        handle.block_on(ctx.inference.complete(CompletionRequest {
                            system_prompt: system.to_string(),
                            user_prompt: format!("File: {}\n\n{}", rel_path, truncated),
                            max_tokens: 1500,
                            temperature: 0.2,
                        }))
                    })
                }
                Err(_) => {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| PhaseError::Io(std::io::Error::other(e)))?;
                    rt.block_on(ctx.inference.complete(CompletionRequest {
                        system_prompt: system.to_string(),
                        user_prompt: format!("File: {}\n\n{}", rel_path, truncated),
                        max_tokens: 1500,
                        temperature: 0.2,
                    }))
                }
            };

            match response {
                Ok(text) => {
                    let json_str = extract_json_array(&text);
                    if let Ok(facts) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                        for fact in &facts {
                            let kind = fact.get("kind").and_then(|v| v.as_str()).unwrap_or("doc_fact");
                            let label = fact.get("label").and_then(|v| v.as_str()).unwrap_or("unknown");
                            let desc = fact.get("description").and_then(|v| v.as_str()).unwrap_or("");

                            ctx.graph.insert_node(&Node {
                                id: None,
                                project_id: ctx.config.project.name.clone(),
                                kind: kind.to_string(),
                                label: label.to_string(),
                                file_path: Some(rel_path.clone()),
                                line_start: None,
                                line_end: None,
                                properties_json: serde_json::json!({
                                    "description": desc,
                                    "source_file": rel_path,
                                }).to_string(),
                                phase_id: PhaseId::DocsIngestion.as_u8(),
                            })?;
                            nodes_created += 1;
                        }
                    }
                }
                Err(e) => {
                    warnings.push(format!("inference failed for {}: {}", rel_path, e));
                }
            }
        }

        ctx.progress.phase_detail(&format!("facts extracted: {}", nodes_created));

        Ok(PhaseOutput {
            nodes_created,
            edges_created: 0,
            warnings,
        })
    }
}

fn extract_json_array(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(start) = trimmed.find('[')
        && let Some(end) = trimmed.rfind(']')
    {
        return &trimmed[start..=end];
    }
    trimmed
}
