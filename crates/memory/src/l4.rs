use std::path::Path;
use rusqlite::{Connection, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectMemoryError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ProjectMemory {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub id: Option<i64>,
    pub project_id: String,
    pub kind: String,
    pub label: String,
    pub description: String,
    pub source: FactSource,
    pub confidence: f32,
    pub access_count: u32,
}

#[derive(Debug, Clone)]
pub enum FactSource {
    Bootstrap,
    Promotion { from_event_id: i64 },
    UserDeclared,
    Inferred,
}

impl FactSource {
    fn as_str(&self) -> &'static str {
        match self {
            FactSource::Bootstrap => "bootstrap",
            FactSource::Promotion { .. } => "promotion",
            FactSource::UserDeclared => "user_declared",
            FactSource::Inferred => "inferred",
        }
    }

    fn event_id(&self) -> Option<i64> {
        match self {
            FactSource::Promotion { from_event_id } => Some(*from_event_id),
            _ => None,
        }
    }
}

impl ProjectMemory {
    pub fn open(path: &Path) -> Result<Self, ProjectMemoryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
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

    pub fn open_in_memory() -> Result<Self, ProjectMemoryError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), ProjectMemoryError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS facts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                label TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL,
                source_event_id INTEGER,
                confidence REAL NOT NULL DEFAULT 0.5,
                access_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tombstoned_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_facts_project_kind
                ON facts(project_id, kind) WHERE tombstoned_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_facts_label
                ON facts(project_id, label) WHERE tombstoned_at IS NULL;"
        )?;
        Ok(())
    }

    pub fn insert_fact(&self, fact: &Fact) -> Result<i64, ProjectMemoryError> {
        self.conn.execute(
            "INSERT INTO facts (project_id, kind, label, description, source, source_event_id, confidence, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                fact.project_id,
                fact.kind,
                fact.label,
                fact.description,
                fact.source.as_str(),
                fact.source.event_id(),
                fact.confidence,
                fact.access_count,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn query_by_kind(&self, project_id: &str, kind: &str) -> Result<Vec<Fact>, ProjectMemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, kind, label, description, source, source_event_id, confidence, access_count
             FROM facts WHERE project_id = ?1 AND kind = ?2 AND tombstoned_at IS NULL"
        )?;
        let rows = stmt.query_map(params![project_id, kind], Self::row_to_fact)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(ProjectMemoryError::from)
    }

    pub fn query_by_label(&self, project_id: &str, label: &str) -> Result<Vec<Fact>, ProjectMemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, kind, label, description, source, source_event_id, confidence, access_count
             FROM facts WHERE project_id = ?1 AND label LIKE ?2 AND tombstoned_at IS NULL"
        )?;
        let pattern = format!("%{}%", label);
        let rows = stmt.query_map(params![project_id, pattern], Self::row_to_fact)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(ProjectMemoryError::from)
    }

    pub fn increment_access(&self, fact_id: i64) -> Result<(), ProjectMemoryError> {
        self.conn.execute(
            "UPDATE facts SET access_count = access_count + 1 WHERE id = ?1",
            params![fact_id],
        )?;
        Ok(())
    }

    pub fn tombstone(&self, fact_id: i64) -> Result<(), ProjectMemoryError> {
        self.conn.execute(
            "UPDATE facts SET tombstoned_at = datetime('now') WHERE id = ?1",
            params![fact_id],
        )?;
        Ok(())
    }

    pub fn count_by_kind(&self, project_id: &str) -> Result<Vec<(String, i64)>, ProjectMemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, COUNT(*) FROM facts
             WHERE project_id = ?1 AND tombstoned_at IS NULL
             GROUP BY kind ORDER BY COUNT(*) DESC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(ProjectMemoryError::from)
    }

    fn row_to_fact(row: &rusqlite::Row) -> rusqlite::Result<Fact> {
        let source_str: String = row.get(5)?;
        let source_event_id: Option<i64> = row.get(6)?;

        let source = match source_str.as_str() {
            "bootstrap" => FactSource::Bootstrap,
            "promotion" => FactSource::Promotion { from_event_id: source_event_id.unwrap_or(0) },
            "user_declared" => FactSource::UserDeclared,
            _ => FactSource::Inferred,
        };

        Ok(Fact {
            id: Some(row.get(0)?),
            project_id: row.get(1)?,
            kind: row.get(2)?,
            label: row.get(3)?,
            description: row.get(4)?,
            source,
            confidence: row.get(7)?,
            access_count: row.get(8)?,
        })
    }
}
