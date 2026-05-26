use crate::context::PipelineContext;
use crate::graph_store::Node;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct GitMiningPhase;

impl Phase for GitMiningPhase {
    fn id(&self) -> PhaseId {
        PhaseId::GitMining
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        let mut warnings = Vec::new();
        let mut nodes_created = 0;

        let repo = match git2::Repository::open(&ctx.root_path) {
            Ok(r) => r,
            Err(e) => {
                return Err(PhaseError::Skipped(format!("not a git repo: {}", e)));
            }
        };

        let mut revwalk = match repo.revwalk() {
            Ok(r) => r,
            Err(e) => {
                warnings.push(format!("cannot walk git history: {}", e));
                return Ok(PhaseOutput { nodes_created: 0, edges_created: 0, warnings });
            }
        };

        if revwalk.push_head().is_err() {
            return Err(PhaseError::Skipped("no HEAD commit".into()));
        }

        let mut file_stats: std::collections::HashMap<String, FileStats> = std::collections::HashMap::new();
        let mut commit_count = 0u64;

        for oid in revwalk.flatten().take(2000) {
            let commit = match repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };
            commit_count += 1;

            let tree = match commit.tree() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

            let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
                Ok(d) => d,
                Err(_) => continue,
            };

            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        let path_str = path.to_string_lossy().replace('\\', "/");
                        if path_str.ends_with(".py") {
                            let entry = file_stats.entry(path_str).or_default();
                            entry.change_count += 1;
                        }
                    }
                    true
                },
                None, None, None,
            ).ok();
        }

        ctx.progress.phase_detail(&format!("commits analyzed: {}", commit_count));

        let mut hotspots: Vec<_> = file_stats.iter()
            .filter(|(_, stats)| stats.change_count > 5)
            .collect();
        hotspots.sort_by_key(|b| std::cmp::Reverse(b.1.change_count));

        for (path, stats) in hotspots.iter().take(50) {
            let existing_nodes = ctx.graph.query_nodes_by_file(&ctx.config.project.name, path)?;
            if existing_nodes.is_empty() {
                ctx.graph.insert_node(&Node {
                    id: None,
                    project_id: ctx.config.project.name.clone(),
                    kind: "hotspot".to_string(),
                    label: path.to_string(),
                    file_path: Some(path.to_string()),
                    line_start: None,
                    line_end: None,
                    properties_json: serde_json::json!({
                        "change_count": stats.change_count,
                        "is_hotspot": true,
                    }).to_string(),
                    phase_id: PhaseId::GitMining.as_u8(),
                })?;
                nodes_created += 1;
            } else {
                for node in &existing_nodes {
                    if let Some(id) = node.id {
                        let mut props: serde_json::Value = serde_json::from_str(&node.properties_json).unwrap_or_default();
                        if let serde_json::Value::Object(ref mut map) = props {
                            map.insert("churn_score".into(), serde_json::json!(stats.change_count));
                            map.insert("is_hotspot".into(), serde_json::json!(stats.change_count > 10));
                        }
                        ctx.graph.update_node_properties(id, &props.to_string())?;
                    }
                }
            }
        }

        ctx.progress.phase_detail(&format!("hotspots detected: {}", hotspots.len().min(50)));

        Ok(PhaseOutput {
            nodes_created,
            edges_created: 0,
            warnings,
        })
    }
}

#[derive(Default)]
struct FileStats {
    change_count: u64,
}
