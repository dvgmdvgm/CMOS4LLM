use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("failed to create database: {0}")]
    CreateFailed(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    L2,
    L3,
}

impl Layer {
    fn as_str(&self) -> &'static str {
        match self {
            Layer::L2 => "L2",
            Layer::L3 => "L3",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "L2" => Some(Layer::L2),
            "L3" => Some(Layer::L3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    Turn,
    Decision,
    ScratchFact,
    Extraction,
    TaskStart,
    TaskEnd,
    Lesson,
    RejectedApproach,
    Promotion,
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::Turn => "turn",
            EventType::Decision => "decision",
            EventType::ScratchFact => "scratch_fact",
            EventType::Extraction => "extraction",
            EventType::TaskStart => "task_start",
            EventType::TaskEnd => "task_end",
            EventType::Lesson => "lesson",
            EventType::RejectedApproach => "rejected_approach",
            EventType::Promotion => "promotion",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "turn" => Some(EventType::Turn),
            "decision" => Some(EventType::Decision),
            "scratch_fact" => Some(EventType::ScratchFact),
            "extraction" => Some(EventType::Extraction),
            "task_start" => Some(EventType::TaskStart),
            "task_end" => Some(EventType::TaskEnd),
            "lesson" => Some(EventType::Lesson),
            "rejected_approach" => Some(EventType::RejectedApproach),
            "promotion" => Some(EventType::Promotion),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub id: Option<i64>,
    pub project_id: String,
    pub layer: Layer,
    pub event_type: EventType,
    pub entity_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: String,
    pub payload: serde_json::Value,
    pub access_count: u32,
    pub importance: f32,
}

pub struct EventStore {
    conn: Connection,
}

impl EventStore {
    pub fn open(path: &std::path::Path) -> Result<Self, EventStoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(EventStoreError::CreateFailed)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;"
        )?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, EventStoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), EventStoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                layer TEXT NOT NULL CHECK(layer IN ('L2', 'L3')),
                event_type TEXT NOT NULL,
                entity_id TEXT,
                session_id TEXT,
                timestamp TEXT NOT NULL,
                payload TEXT NOT NULL DEFAULT '{}',
                access_count INTEGER NOT NULL DEFAULT 0,
                importance REAL NOT NULL DEFAULT 0.5,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_events_project_time
                ON events(project_id, timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_entity
                ON events(project_id, entity_id) WHERE entity_id IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_events_layer
                ON events(project_id, layer);
            CREATE INDEX IF NOT EXISTS idx_events_session
                ON events(project_id, session_id) WHERE session_id IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_events_type
                ON events(project_id, event_type);"
        )?;
        Ok(())
    }

    pub fn append(&self, event: &MemoryEvent) -> Result<i64, EventStoreError> {
        self.conn.execute(
            "INSERT INTO events (project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.project_id,
                event.layer.as_str(),
                event.event_type.as_str(),
                event.entity_id,
                event.session_id,
                event.timestamp,
                event.payload.to_string(),
                event.access_count,
                event.importance,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn query_by_time_range(
        &self,
        project_id: &str,
        from: &str,
        to: &str,
    ) -> Result<Vec<MemoryEvent>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance
             FROM events
             WHERE project_id = ?1 AND timestamp BETWEEN ?2 AND ?3
             ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map(params![project_id, from, to], Self::row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    pub fn query_by_session(
        &self,
        project_id: &str,
        session_id: &str,
    ) -> Result<Vec<MemoryEvent>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance
             FROM events
             WHERE project_id = ?1 AND session_id = ?2
             ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map(params![project_id, session_id], Self::row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    pub fn query_by_entity(
        &self,
        project_id: &str,
        entity_id: &str,
    ) -> Result<Vec<MemoryEvent>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance
             FROM events
             WHERE project_id = ?1 AND entity_id = ?2
             ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map(params![project_id, entity_id], Self::row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    pub fn query_by_layer(
        &self,
        project_id: &str,
        layer: Layer,
    ) -> Result<Vec<MemoryEvent>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance
             FROM events
             WHERE project_id = ?1 AND layer = ?2
             ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map(params![project_id, layer.as_str()], Self::row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    pub fn query_by_type(
        &self,
        project_id: &str,
        event_type: EventType,
    ) -> Result<Vec<MemoryEvent>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance
             FROM events
             WHERE project_id = ?1 AND event_type = ?2
             ORDER BY timestamp ASC"
        )?;
        let rows = stmt.query_map(params![project_id, event_type.as_str()], Self::row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    pub fn increment_access(&self, event_id: i64) -> Result<(), EventStoreError> {
        self.conn.execute(
            "UPDATE events SET access_count = access_count + 1 WHERE id = ?1",
            params![event_id],
        )?;
        Ok(())
    }

    pub fn promote_to_l3(&self, event_id: i64) -> Result<(), EventStoreError> {
        self.conn.execute(
            "UPDATE events SET layer = 'L3' WHERE id = ?1 AND layer = 'L2'",
            params![event_id],
        )?;
        Ok(())
    }

    pub fn candidates_for_promotion(
        &self,
        project_id: &str,
        min_access_count: u32,
        min_importance: f32,
    ) -> Result<Vec<MemoryEvent>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, layer, event_type, entity_id, session_id, timestamp, payload, access_count, importance
             FROM events
             WHERE project_id = ?1 AND layer = 'L2' AND access_count >= ?2 AND importance >= ?3
             ORDER BY importance DESC, access_count DESC"
        )?;
        let rows = stmt.query_map(params![project_id, min_access_count, min_importance], Self::row_to_event)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    pub fn count_by_layer(&self, project_id: &str) -> Result<Vec<(String, i64)>, EventStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT layer, COUNT(*) FROM events WHERE project_id = ?1 GROUP BY layer"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(EventStoreError::from)
    }

    fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<MemoryEvent> {
        let layer_str: String = row.get(2)?;
        let type_str: String = row.get(3)?;
        let payload_str: String = row.get(7)?;

        Ok(MemoryEvent {
            id: Some(row.get(0)?),
            project_id: row.get(1)?,
            layer: Layer::from_str(&layer_str).unwrap_or(Layer::L2),
            event_type: EventType::from_str(&type_str).unwrap_or(EventType::Turn),
            entity_id: row.get(4)?,
            session_id: row.get(5)?,
            timestamp: row.get(6)?,
            payload: serde_json::from_str(&payload_str).unwrap_or_default(),
            access_count: row.get(8)?,
            importance: row.get(9)?,
        })
    }
}
