use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::schema::*;
use rust_mcp_sdk::McpServer;

use cmos_memory::l1::{SlotPriority, WorkingMemory};
use cmos_memory::l2l3::{EventStore, EventType, Layer};
use cmos_memory::l4::ProjectMemory;
use cmos_retrieval::{ContextAssembler, ContextQuery, EmbeddingClient, EmbeddingConfig, VectorIndex};

use crate::analytics::TokenTracker;
use crate::tools;

pub struct CmosState {
    pub working_memory: WorkingMemory,
    pub data_root: PathBuf,
    pub token_tracker: TokenTracker,
}

impl CmosState {
    fn event_store(&self) -> std::result::Result<EventStore, String> {
        let path = self.data_root.join("events.db");
        EventStore::open(&path).map_err(|e| format!("Failed to open event store: {e}"))
    }

    fn project_memory(&self) -> std::result::Result<ProjectMemory, String> {
        let path = self.data_root.join("facts.db");
        ProjectMemory::open(&path).map_err(|e| format!("Failed to open project memory: {e}"))
    }

    fn vector_index(&self) -> Option<VectorIndex> {
        let path = self.data_root.join("vectors");
        VectorIndex::open(&path, 768).ok()
    }

    fn embedding_client(&self) -> EmbeddingClient {
        EmbeddingClient::new(EmbeddingConfig::default())
    }
}

pub struct CmosHandler {
    pub state: Arc<CmosState>,
}

fn text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::TextContent(TextContent::new(text, None, None))],
        is_error: None,
        meta: None,
        structured_content: None,
    }
}

fn error_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::TextContent(TextContent::new(text, None, None))],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

fn args_to_value(args: &Option<serde_json::Map<String, serde_json::Value>>) -> Option<serde_json::Value> {
    args.as_ref().map(|m| serde_json::Value::Object(m.clone()))
}

#[async_trait]
impl ServerHandler for CmosHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: tools::tool_definitions(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let args = args_to_value(&params.arguments);
        let result = match params.name.as_str() {
            "cmos_read_memory" => self.handle_read_memory(&args).await,
            "cmos_write_memory" => self.handle_write_memory(&args).await,
            "cmos_query_memory" => self.handle_query_memory(&args).await,
            "cmos_assemble_context" => self.handle_assemble_context(&args).await,
            "cmos_search_similar" => self.handle_search_similar(&args).await,
            "cmos_memory_stats" => self.handle_memory_stats(&args).await,
            _ => return Err(CallToolError::unknown_tool(&params.name)),
        };

        match result {
            Ok(text) => Ok(text_result(text)),
            Err(e) => Ok(error_result(e)),
        }
    }
}

impl CmosHandler {
    async fn handle_read_memory(
        &self,
        args: &Option<serde_json::Value>,
    ) -> std::result::Result<String, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        let params: tools::ReadMemoryParams =
            serde_json::from_value(args.clone()).map_err(|e| format!("Invalid params: {e}"))?;

        match self.state.working_memory.get(&params.slot_id) {
            Some(slot) => Ok(serde_json::json!({
                "id": slot.id,
                "content": slot.content,
                "priority": format!("{:?}", slot.priority),
                "token_estimate": slot.token_estimate,
                "access_count": slot.access_count,
            })
            .to_string()),
            None => Err(format!("Slot '{}' not found in L1", params.slot_id)),
        }
    }

    async fn handle_write_memory(
        &self,
        args: &Option<serde_json::Value>,
    ) -> std::result::Result<String, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        let params: tools::WriteMemoryParams =
            serde_json::from_value(args.clone()).map_err(|e| format!("Invalid params: {e}"))?;

        let priority = match params.priority.as_str() {
            "scratch" => SlotPriority::Scratch,
            "context" => SlotPriority::Context,
            "policy" => SlotPriority::Policy,
            "system" => SlotPriority::System,
            _ => SlotPriority::Context,
        };

        let evicted = self
            .state
            .working_memory
            .insert(&params.slot_id, &params.content, priority);

        let mut response = serde_json::json!({
            "status": "written",
            "slot_id": params.slot_id,
            "total_tokens": self.state.working_memory.total_tokens(),
            "slot_count": self.state.working_memory.slot_count(),
        });

        if let Some(evicted_slots) = evicted {
            response["evicted"] = serde_json::json!(
                evicted_slots.iter().map(|s| &s.id).collect::<Vec<_>>()
            );
        }

        Ok(response.to_string())
    }

    async fn handle_query_memory(
        &self,
        args: &Option<serde_json::Value>,
    ) -> std::result::Result<String, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        let params: tools::QueryMemoryParams =
            serde_json::from_value(args.clone()).map_err(|e| format!("Invalid params: {e}"))?;

        let layer = params.layer.as_deref().unwrap_or("L4");

        match layer {
            "L4" => {
                let pm = self.state.project_memory()?;
                let facts = if let Some(kind) = &params.kind {
                    pm.query_by_kind(&params.project_id, kind)
                        .map_err(|e| e.to_string())?
                } else if let Some(label) = &params.label {
                    pm.query_by_label(&params.project_id, label)
                        .map_err(|e| e.to_string())?
                } else {
                    let mut all = Vec::new();
                    for kind in &["decision", "policy", "convention", "lesson", "constraint"] {
                        let mut facts = pm
                            .query_by_kind(&params.project_id, kind)
                            .map_err(|e| e.to_string())?;
                        all.append(&mut facts);
                    }
                    all
                };

                let limited: Vec<_> = facts.into_iter().take(params.limit).collect();
                let output: Vec<serde_json::Value> = limited
                    .iter()
                    .map(|f| {
                        serde_json::json!({
                            "id": f.id,
                            "kind": f.kind,
                            "label": f.label,
                            "description": f.description,
                            "confidence": f.confidence,
                            "access_count": f.access_count,
                        })
                    })
                    .collect();
                Ok(serde_json::to_string_pretty(&output).unwrap())
            }
            "L2" | "L3" => {
                let es = self.state.event_store()?;
                let target_layer = if layer == "L2" { Layer::L2 } else { Layer::L3 };

                let events = if let Some(et) = &params.event_type {
                    let event_type = parse_event_type(et)?;
                    es.query_by_type(&params.project_id, event_type)
                        .map_err(|e| e.to_string())?
                } else {
                    es.query_by_layer(&params.project_id, target_layer)
                        .map_err(|e| e.to_string())?
                };

                let limited: Vec<_> = events.into_iter().rev().take(params.limit).collect();
                let output: Vec<serde_json::Value> = limited
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "event_type": format!("{:?}", e.event_type),
                            "layer": format!("{:?}", e.layer),
                            "timestamp": e.timestamp,
                            "entity_id": e.entity_id,
                            "importance": e.importance,
                            "payload": e.payload,
                        })
                    })
                    .collect();
                Ok(serde_json::to_string_pretty(&output).unwrap())
            }
            _ => Err(format!("Unknown layer: {layer}. Use L2, L3, or L4.")),
        }
    }

    async fn handle_assemble_context(
        &self,
        args: &Option<serde_json::Value>,
    ) -> std::result::Result<String, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        let params: tools::AssembleContextParams =
            serde_json::from_value(args.clone()).map_err(|e| format!("Invalid params: {e}"))?;

        let query = ContextQuery {
            project_id: params.project_id.clone(),
            task_description: params.task_description.clone(),
            session_id: params.session_id,
            max_tokens: params.max_tokens,
            include_l1: true,
            include_l3: true,
            include_l4: true,
        };

        let assembler = ContextAssembler::default();
        let es = self.state.event_store().ok();
        let pm = self.state.project_memory().ok();
        let vi = self.state.vector_index();

        let result = if let Some(ref vi) = vi {
            let ec = self.state.embedding_client();
            let query_embedding = ec
                .embed_single(&query.task_description)
                .await
                .map_err(|e| e.to_string())?;

            assembler
                .assemble_hybrid_with_embedding(
                    &query,
                    Some(&self.state.working_memory),
                    es.as_ref(),
                    pm.as_ref(),
                    vi,
                    &query_embedding,
                )
                .map_err(|e| e.to_string())?
        } else {
            assembler
                .assemble(
                    &query,
                    Some(&self.state.working_memory),
                    es.as_ref(),
                    pm.as_ref(),
                )
                .map_err(|e| e.to_string())?
        };

        // Baseline estimate: without CMOS, the full task description + all considered items
        // would be sent raw. Conservative estimate: items_considered * avg_tokens_per_item.
        let baseline_estimate = (result.items_considered as u64) * 200 + (params.task_description.len() as u64 / 4);
        let assembled_tokens = result.total_tokens as u64;

        self.state.token_tracker.record(
            &params.project_id,
            "assemble_context",
            assembled_tokens,
            baseline_estimate.max(assembled_tokens),
        );

        Ok(serde_json::json!({
            "context": result.render_with_header(&params.task_description),
            "total_tokens": result.total_tokens,
            "budget": result.budget,
            "items_considered": result.items_considered,
            "items_included": result.items_included,
            "sections": result.sections.iter().map(|s| serde_json::json!({
                "label": s.label,
                "source_layer": s.source_layer,
                "token_count": s.token_count,
            })).collect::<Vec<_>>(),
        })
        .to_string())
    }

    async fn handle_search_similar(
        &self,
        args: &Option<serde_json::Value>,
    ) -> std::result::Result<String, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        let params: tools::SearchSimilarParams =
            serde_json::from_value(args.clone()).map_err(|e| format!("Invalid params: {e}"))?;

        let vi = self
            .state
            .vector_index()
            .ok_or("Vector index not found. Run `cmos vector index` first.")?;
        let ec = self.state.embedding_client();

        let embedding = ec
            .embed_single(&params.query)
            .await
            .map_err(|e| format!("Embedding failed: {e}"))?;

        let layer_filter = params.layer.as_deref();
        let results = vi
            .search(&embedding, params.limit, layer_filter)
            .map_err(|e| format!("Search failed: {e}"))?;

        let output: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "source_id": r.source_id,
                    "layer": r.layer,
                    "content": r.content,
                    "score": r.distance,
                })
            })
            .collect();
        Ok(serde_json::to_string_pretty(&output).unwrap())
    }

    async fn handle_memory_stats(
        &self,
        args: &Option<serde_json::Value>,
    ) -> std::result::Result<String, String> {
        let args = args.as_ref().ok_or("Missing arguments")?;
        let params: tools::MemoryStatsParams =
            serde_json::from_value(args.clone()).map_err(|e| format!("Invalid params: {e}"))?;

        let mut stats = serde_json::json!({
            "l1": {
                "slot_count": self.state.working_memory.slot_count(),
                "total_tokens": self.state.working_memory.total_tokens(),
            }
        });

        if let Ok(es) = self.state.event_store() {
            let counts = es
                .count_by_layer(&params.project_id)
                .unwrap_or_default();
            stats["l2_l3"] = serde_json::json!({
                "counts_by_layer": counts.iter().map(|(l, c)| serde_json::json!({"layer": l, "count": c})).collect::<Vec<_>>(),
            });
        }

        if let Ok(pm) = self.state.project_memory() {
            let counts = pm
                .count_by_kind(&params.project_id)
                .unwrap_or_default();
            stats["l4"] = serde_json::json!({
                "counts_by_kind": counts.iter().map(|(k, c)| serde_json::json!({"kind": k, "count": c})).collect::<Vec<_>>(),
            });
        }

        if let Some(vi) = self.state.vector_index() {
            stats["vector_index"] = serde_json::json!({
                "total_vectors": vi.count(),
            });
        }

        Ok(serde_json::to_string_pretty(&stats).unwrap())
    }
}

fn parse_event_type(s: &str) -> std::result::Result<EventType, String> {
    match s {
        "turn" => Ok(EventType::Turn),
        "decision" => Ok(EventType::Decision),
        "scratch_fact" => Ok(EventType::ScratchFact),
        "extraction" => Ok(EventType::Extraction),
        "task_start" => Ok(EventType::TaskStart),
        "task_end" => Ok(EventType::TaskEnd),
        "lesson" => Ok(EventType::Lesson),
        "rejected_approach" => Ok(EventType::RejectedApproach),
        "promotion" => Ok(EventType::Promotion),
        _ => Err(format!("Unknown event type: {s}")),
    }
}
