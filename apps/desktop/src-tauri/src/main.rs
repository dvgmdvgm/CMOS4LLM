#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use cmos_memory::l1::{WorkingMemory, WorkingMemoryConfig};
use cmos_memory::l2l3::{EventStore, Layer};
use cmos_memory::l4::ProjectMemory;
use cmos_retrieval::VectorIndex;

struct AppState {
    working_memory: WorkingMemory,
    data_root: PathBuf,
}

impl AppState {
    fn event_store(&self) -> Result<EventStore, String> {
        let path = self.data_root.join("events.db");
        EventStore::open(&path).map_err(|e| format!("{e}"))
    }

    fn project_memory(&self) -> Result<ProjectMemory, String> {
        let path = self.data_root.join("facts.db");
        ProjectMemory::open(&path).map_err(|e| format!("{e}"))
    }

    fn vector_index(&self) -> Option<VectorIndex> {
        let path = self.data_root.join("vectors");
        VectorIndex::open(&path, 768).ok()
    }
}

#[derive(Serialize)]
struct MemoryStats {
    l1_slots: usize,
    l1_tokens: usize,
    l2_count: i64,
    l3_count: i64,
    l4_count: i64,
    vector_count: usize,
}

#[derive(Serialize)]
struct ProjectInfo {
    id: String,
    l4_facts: i64,
    l2_events: i64,
    l3_events: i64,
}

#[derive(Serialize)]
struct FactEntry {
    id: i64,
    kind: String,
    label: String,
    description: String,
    confidence: f32,
    access_count: u32,
}

#[derive(Serialize)]
struct EventEntry {
    id: i64,
    event_type: String,
    layer: String,
    timestamp: String,
    entity_id: Option<String>,
    importance: f32,
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct TokenStats {
    total_requests: u64,
    tokens_saved: u64,
    tokens_without_cmos: u64,
    tokens_with_cmos: u64,
    savings_ratio: f64,
}

#[tauri::command]
fn get_version() -> String {
    format!("CMOS v{}", cmos_core::version())
}

#[tauri::command]
fn get_memory_stats(project_id: String, state: State<Arc<AppState>>) -> Result<MemoryStats, String> {
    let mut stats = MemoryStats {
        l1_slots: state.working_memory.slot_count(),
        l1_tokens: state.working_memory.total_tokens(),
        l2_count: 0,
        l3_count: 0,
        l4_count: 0,
        vector_count: 0,
    };

    if let Ok(es) = state.event_store() {
        let counts = es.count_by_layer(&project_id).unwrap_or_default();
        for (layer, count) in &counts {
            match layer.as_str() {
                "L2" => stats.l2_count = *count,
                "L3" => stats.l3_count = *count,
                _ => {}
            }
        }
    }

    if let Ok(pm) = state.project_memory() {
        let counts = pm.count_by_kind(&project_id).unwrap_or_default();
        stats.l4_count = counts.iter().map(|(_, c)| c).sum();
    }

    if let Some(vi) = state.vector_index() {
        stats.vector_count = vi.count();
    }

    Ok(stats)
}

#[tauri::command]
fn list_projects(state: State<Arc<AppState>>) -> Result<Vec<ProjectInfo>, String> {
    let mut projects: Vec<ProjectInfo> = Vec::new();

    let events_path = state.data_root.join("events.db");
    if let Some(Ok(conn)) = events_path.exists().then(|| rusqlite::Connection::open(&events_path)) {
        let mut stmt = conn
            .prepare("SELECT DISTINCT project_id FROM events")
            .map_err(|e| format!("{e}"))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("{e}"))?;

        for pid in rows.flatten() {
            let mut info = ProjectInfo {
                id: pid.clone(),
                l4_facts: 0,
                l2_events: 0,
                l3_events: 0,
            };

            if let Ok(es) = state.event_store() {
                let counts = es.count_by_layer(&pid).unwrap_or_default();
                for (layer, count) in &counts {
                    match layer.as_str() {
                        "L2" => info.l2_events = *count,
                        "L3" => info.l3_events = *count,
                        _ => {}
                    }
                }
            }

            if let Ok(pm) = state.project_memory() {
                let counts = pm.count_by_kind(&pid).unwrap_or_default();
                info.l4_facts = counts.iter().map(|(_, c)| c).sum();
            }

            projects.push(info);
        }
    }

    let facts_path = state.data_root.join("facts.db");
    if let Some(Ok(conn)) = facts_path.exists().then(|| rusqlite::Connection::open(&facts_path)) {
        let mut stmt = conn
            .prepare("SELECT DISTINCT project_id FROM facts WHERE tombstoned_at IS NULL")
            .map_err(|e| format!("{e}"))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("{e}"))?;

        for pid in rows.flatten() {
            if !projects.iter().any(|p| p.id == pid) {
                let mut info = ProjectInfo {
                    id: pid.clone(),
                    l4_facts: 0,
                    l2_events: 0,
                    l3_events: 0,
                };
                if let Ok(pm) = state.project_memory() {
                    let counts = pm.count_by_kind(&pid).unwrap_or_default();
                    info.l4_facts = counts.iter().map(|(_, c)| c).sum();
                }
                projects.push(info);
            }
        }
    }

    Ok(projects)
}

#[tauri::command]
fn get_facts(
    project_id: String,
    kind: Option<String>,
    limit: Option<usize>,
    state: State<Arc<AppState>>,
) -> Result<Vec<FactEntry>, String> {
    let pm = state.project_memory()?;
    let limit = limit.unwrap_or(50);

    let facts = if let Some(kind) = kind {
        pm.query_by_kind(&project_id, &kind).map_err(|e| format!("{e}"))?
    } else {
        let mut all = Vec::new();
        for k in &["decision", "policy", "convention", "lesson", "constraint"] {
            let mut f = pm.query_by_kind(&project_id, k).map_err(|e| format!("{e}"))?;
            all.append(&mut f);
        }
        all
    };

    Ok(facts
        .into_iter()
        .take(limit)
        .map(|f| FactEntry {
            id: f.id.unwrap_or(0),
            kind: f.kind,
            label: f.label,
            description: f.description,
            confidence: f.confidence,
            access_count: f.access_count,
        })
        .collect())
}

#[tauri::command]
fn get_events(
    project_id: String,
    layer: Option<String>,
    limit: Option<usize>,
    state: State<Arc<AppState>>,
) -> Result<Vec<EventEntry>, String> {
    let es = state.event_store()?;
    let limit = limit.unwrap_or(50);

    let target_layer = match layer.as_deref() {
        Some("L2") => Some(Layer::L2),
        Some("L3") => Some(Layer::L3),
        _ => None,
    };

    let events = if let Some(l) = target_layer {
        es.query_by_layer(&project_id, l).map_err(|e| format!("{e}"))?
    } else {
        let mut all = es.query_by_layer(&project_id, Layer::L2).map_err(|e| format!("{e}"))?;
        let mut l3 = es.query_by_layer(&project_id, Layer::L3).map_err(|e| format!("{e}"))?;
        all.append(&mut l3);
        all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all
    };

    Ok(events
        .into_iter()
        .rev()
        .take(limit)
        .map(|e| EventEntry {
            id: e.id.unwrap_or(0),
            event_type: format!("{:?}", e.event_type),
            layer: format!("{:?}", e.layer),
            timestamp: e.timestamp,
            entity_id: e.entity_id,
            importance: e.importance,
            payload: e.payload,
        })
        .collect())
}

#[tauri::command]
fn get_token_stats(state: State<Arc<AppState>>) -> TokenStats {
    let db_path = state.data_root.join("token_analytics.db");
    if !db_path.exists() {
        return TokenStats {
            total_requests: 0,
            tokens_saved: 0,
            tokens_without_cmos: 0,
            tokens_with_cmos: 0,
            savings_ratio: 0.0,
        };
    }

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return TokenStats {
            total_requests: 0,
            tokens_saved: 0,
            tokens_without_cmos: 0,
            tokens_with_cmos: 0,
            savings_ratio: 0.0,
        },
    };

    let result = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(tokens_assembled), 0), COALESCE(SUM(tokens_baseline_estimate), 0) FROM token_events",
        [],
        |row| Ok((row.get::<_, u64>(0).unwrap_or(0), row.get::<_, u64>(1).unwrap_or(0), row.get::<_, u64>(2).unwrap_or(0))),
    );

    match result {
        Ok((total_requests, tokens_with_cmos, tokens_without_cmos)) => {
            let tokens_saved = tokens_without_cmos.saturating_sub(tokens_with_cmos);
            let savings_ratio = if tokens_with_cmos > 0 {
                tokens_without_cmos as f64 / tokens_with_cmos as f64
            } else {
                0.0
            };
            TokenStats {
                total_requests,
                tokens_saved,
                tokens_without_cmos,
                tokens_with_cmos,
                savings_ratio,
            }
        }
        Err(_) => TokenStats {
            total_requests: 0,
            tokens_saved: 0,
            tokens_without_cmos: 0,
            tokens_with_cmos: 0,
            savings_ratio: 0.0,
        },
    }
}

fn resolve_data_root() -> PathBuf {
    if let Ok(dir) = std::env::var("CMOS_DATA_DIR") {
        return PathBuf::from(dir);
    }

    // Portable: store data next to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.join("data");
        }
    }

    PathBuf::from("data")
}

fn main() {
    tracing_subscriber::fmt::init();

    let data_root = resolve_data_root();
    std::fs::create_dir_all(&data_root).expect("failed to create data directory");

    let app_state = Arc::new(AppState {
        working_memory: WorkingMemory::new(WorkingMemoryConfig::default()),
        data_root,
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_version,
            get_memory_stats,
            list_projects,
            get_facts,
            get_events,
            get_token_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CMOS desktop");
}
