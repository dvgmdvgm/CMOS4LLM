use std::path::Path;
use crate::context::PipelineContext;
use crate::extractors::django::DjangoExtractor;
use crate::graph_store::Edge;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct SchemaExtractionPhase;

impl Phase for SchemaExtractionPhase {
    fn id(&self) -> PhaseId {
        PhaseId::SchemaExtraction
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let django = DjangoExtractor::new();
        let mut edges_created = 0;
        let nodes_created = 0;
        let mut warnings = Vec::new();

        let model_nodes = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "django_model")?;

        let model_files: Vec<String> = model_nodes.iter()
            .filter_map(|n| n.file_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for rel_path in &model_files {
            let full_path = ctx.root_path.join(rel_path.replace('/', "\\"));
            let source = match std::fs::read(&full_path) {
                Ok(s) => s,
                Err(e) => {
                    warnings.push(format!("cannot read {}: {}", rel_path, e));
                    continue;
                }
            };

            let fields = match django.extract_model_fields(&source, Path::new(rel_path)) {
                Ok(f) => f,
                Err(e) => {
                    warnings.push(format!("field extraction error in {}: {}", rel_path, e));
                    continue;
                }
            };

            for field in &fields {
                if let Some(ref target) = field.relation_target {
                    let edge_kind = if field.field_type.contains("ManyToMany") {
                        "m2m_to"
                    } else {
                        "fk_to"
                    };

                    let source_id = ctx.graph.find_node_id_by_label(
                        &ctx.config.project.name, "django_model", &field.model_name
                    )?;
                    let target_id = ctx.graph.find_node_id_by_label(
                        &ctx.config.project.name, "django_model", target
                    )?;

                    if let (Some(src), Some(tgt)) = (source_id, target_id) {
                        ctx.graph.insert_edge(&Edge {
                            id: None,
                            project_id: ctx.config.project.name.clone(),
                            source_id: src,
                            target_id: tgt,
                            kind: edge_kind.to_string(),
                            properties_json: serde_json::json!({
                                "field_name": field.field_name,
                                "field_type": field.field_type,
                            }).to_string(),
                            phase_id: PhaseId::SchemaExtraction.as_u8(),
                        })?;
                        edges_created += 1;
                    }
                }
            }
        }

        ctx.progress.phase_detail(&format!("relationships: {}", edges_created));

        Ok(PhaseOutput {
            nodes_created,
            edges_created,
            warnings,
        })
    }
}
