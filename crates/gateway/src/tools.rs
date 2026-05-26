use std::collections::BTreeMap;

use rust_mcp_sdk::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadMemoryParams {
    pub project_id: String,
    pub slot_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WriteMemoryParams {
    pub project_id: String,
    pub slot_id: String,
    pub content: String,
    #[serde(default = "default_priority")]
    pub priority: String,
}

fn default_priority() -> String {
    "context".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryMemoryParams {
    pub project_id: String,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssembleContextParams {
    pub project_id: String,
    pub task_description: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default = "default_budget")]
    pub max_tokens: usize,
}

fn default_budget() -> usize {
    32_000
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchSimilarParams {
    pub project_id: String,
    pub query: String,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

fn default_search_limit() -> usize {
    10
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryStatsParams {
    pub project_id: String,
}

fn make_prop(
    entries: &[(&str, &str, &str)],
) -> BTreeMap<String, serde_json::Map<String, serde_json::Value>> {
    let mut map = BTreeMap::new();
    for (name, type_str, desc) in entries {
        let mut prop = serde_json::Map::new();
        prop.insert(
            "type".into(),
            serde_json::Value::String(type_str.to_string()),
        );
        prop.insert(
            "description".into(),
            serde_json::Value::String(desc.to_string()),
        );
        map.insert(name.to_string(), prop);
    }
    map
}

fn make_tool(name: &str, description: &str, required: Vec<String>, props: &[(&str, &str, &str)]) -> Tool {
    Tool {
        name: name.into(),
        description: Some(description.into()),
        input_schema: ToolInputSchema::new(required, Some(make_prop(props)), None),
        annotations: None,
        execution: None,
        icons: vec![],
        meta: None,
        output_schema: None,
        title: None,
    }
}

pub fn tool_definitions() -> Vec<Tool> {
    vec![
        make_tool(
            "cmos_read_memory",
            "Read a specific slot from L1 working memory by its ID.",
            vec!["project_id".into(), "slot_id".into()],
            &[
                ("project_id", "string", "Project identifier"),
                ("slot_id", "string", "Memory slot ID to read"),
            ],
        ),
        make_tool(
            "cmos_write_memory",
            "Write content to L1 working memory. Evicts lowest-priority slots if budget exceeded.",
            vec!["project_id".into(), "slot_id".into(), "content".into()],
            &[
                ("project_id", "string", "Project identifier"),
                ("slot_id", "string", "Unique slot ID"),
                ("content", "string", "Content to store"),
                ("priority", "string", "Slot priority: scratch, context, policy, system (default: context)"),
            ],
        ),
        make_tool(
            "cmos_query_memory",
            "Query memory events from L2/L3 (episodes) or facts from L4 (project knowledge).",
            vec!["project_id".into()],
            &[
                ("project_id", "string", "Project identifier"),
                ("layer", "string", "Memory layer: L2, L3, or L4"),
                ("kind", "string", "Fact kind filter (L4 only): decision, policy, convention, lesson, constraint"),
                ("label", "string", "Label substring search (L4 only)"),
                ("event_type", "string", "Event type filter (L2/L3): turn, decision, extraction, lesson, rejected_approach"),
                ("limit", "integer", "Max results (default: 20)"),
            ],
        ),
        make_tool(
            "cmos_assemble_context",
            "Assemble optimized context from all memory layers for a given task. Uses hybrid retrieval (vector + keyword) when available.",
            vec!["project_id".into(), "task_description".into()],
            &[
                ("project_id", "string", "Project identifier"),
                ("task_description", "string", "Description of the current task (used for relevance scoring)"),
                ("session_id", "string", "Optional session ID to scope L3 retrieval"),
                ("max_tokens", "integer", "Token budget (default: 32000)"),
            ],
        ),
        make_tool(
            "cmos_search_similar",
            "Semantic similarity search across memory using vector embeddings. Requires Ollama with an embedding model.",
            vec!["project_id".into(), "query".into()],
            &[
                ("project_id", "string", "Project identifier"),
                ("query", "string", "Natural language query for semantic search"),
                ("layer", "string", "Restrict search to L3 or L4"),
                ("limit", "integer", "Max results (default: 10)"),
            ],
        ),
        make_tool(
            "cmos_memory_stats",
            "Get memory statistics: slot count/tokens in L1, event counts in L2/L3, fact counts in L4.",
            vec!["project_id".into()],
            &[
                ("project_id", "string", "Project identifier"),
            ],
        ),
    ]
}
