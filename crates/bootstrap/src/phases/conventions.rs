use crate::context::PipelineContext;
use crate::graph_store::Node;
use crate::inference::CompletionRequest;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct ConventionMiningPhase;

impl Phase for ConventionMiningPhase {
    fn id(&self) -> PhaseId {
        PhaseId::ConventionMining
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let mut nodes_created = 0;
        let mut warnings = Vec::new();

        let functions = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "function")?;
        let views = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "django_view")?;
        let models = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "django_model")?;

        let total_functions = functions.len();
        let total_views = views.len();
        let total_models = models.len();

        let fbv_count = views.iter()
            .filter(|v| {
                let props: serde_json::Value = serde_json::from_str(&v.properties_json).unwrap_or_default();
                props.get("decorators").is_some()
            })
            .count();
        let cbv_count = total_views - fbv_count;

        let fn_names: Vec<&str> = functions.iter()
            .take(50)
            .map(|f| f.label.as_str())
            .collect();

        let naming_sample = fn_names.join(", ");

        let system = "You are a code convention analyzer. Analyze the given code statistics and naming samples. Output a JSON array of conventions found. Each convention: {\"name\": \"short_id\", \"description\": \"what the convention is\", \"confidence\": 0.0-1.0}";
        let prompt = format!(
            "Project statistics:\n- Total functions: {}\n- Total views: {} (FBV: {}, CBV: {})\n- Total models: {}\n\nFunction name samples: {}\n\nIdentify naming conventions, architectural patterns, and style preferences.",
            total_functions, total_views, fbv_count, cbv_count, total_models, naming_sample
        );

        let rt = tokio::runtime::Handle::try_current();
        let response = match rt {
            Ok(handle) => {
                tokio::task::block_in_place(|| {
                    handle.block_on(ctx.inference.complete(CompletionRequest {
                        system_prompt: system.to_string(),
                        user_prompt: prompt,
                        max_tokens: 1000,
                        temperature: 0.3,
                    }))
                })
            }
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| PhaseError::Io(std::io::Error::other(e)))?;
                rt.block_on(ctx.inference.complete(CompletionRequest {
                    system_prompt: system.to_string(),
                    user_prompt: prompt,
                    max_tokens: 1000,
                    temperature: 0.3,
                }))
            }
        };

        match response {
            Ok(text) => {
                let json_str = extract_json_array(&text);
                if let Ok(conventions) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    for conv in &conventions {
                        let name = conv.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let desc = conv.get("description").and_then(|v| v.as_str()).unwrap_or("");
                        let confidence = conv.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);

                        ctx.graph.insert_node(&Node {
                            id: None,
                            project_id: ctx.config.project.name.clone(),
                            kind: "convention".to_string(),
                            label: name.to_string(),
                            file_path: None,
                            line_start: None,
                            line_end: None,
                            properties_json: serde_json::json!({
                                "description": desc,
                                "confidence": confidence,
                            }).to_string(),
                            phase_id: PhaseId::ConventionMining.as_u8(),
                        })?;
                        nodes_created += 1;
                    }
                    ctx.progress.phase_detail(&format!("conventions detected: {}", conventions.len()));
                } else {
                    warnings.push("failed to parse LM response as JSON array".into());
                }
            }
            Err(e) => {
                warnings.push(format!("inference failed: {}", e));
                ctx.progress.phase_warning(&format!("inference failed: {}", e));
            }
        }

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
