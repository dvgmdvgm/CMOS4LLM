use rusqlite::{Connection, params};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("failed to create database at {0}: {1}")]
    CreateFailed(String, std::io::Error),
}

pub struct GraphStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Option<i64>,
    pub project_id: String,
    pub kind: String,
    pub label: String,
    pub file_path: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub properties_json: String,
    pub phase_id: u8,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: Option<i64>,
    pub project_id: String,
    pub source_id: i64,
    pub target_id: i64,
    pub kind: String,
    pub properties_json: String,
    pub phase_id: u8,
}

impl GraphStore {
    pub fn open(path: &Path) -> Result<Self, GraphError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GraphError::CreateFailed(parent.display().to_string(), e))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, GraphError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), GraphError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                config_json TEXT
            );

            CREATE TABLE IF NOT EXISTS nodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL REFERENCES projects(id),
                kind TEXT NOT NULL,
                label TEXT NOT NULL,
                file_path TEXT,
                line_start INTEGER,
                line_end INTEGER,
                properties_json TEXT NOT NULL DEFAULT '{}',
                phase_id INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tombstoned_at TEXT,
                supersedes INTEGER REFERENCES nodes(id)
            );

            CREATE TABLE IF NOT EXISTS edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL REFERENCES projects(id),
                source_id INTEGER NOT NULL REFERENCES nodes(id),
                target_id INTEGER NOT NULL REFERENCES nodes(id),
                kind TEXT NOT NULL,
                properties_json TEXT DEFAULT '{}',
                phase_id INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                tombstoned_at TEXT
            );

            CREATE TABLE IF NOT EXISTS pipeline_checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL REFERENCES projects(id),
                phase_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                stats_json TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_nodes_project_kind
                ON nodes(project_id, kind) WHERE tombstoned_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_nodes_file
                ON nodes(project_id, file_path) WHERE tombstoned_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_edges_source
                ON edges(source_id, kind) WHERE tombstoned_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_edges_target
                ON edges(target_id, kind) WHERE tombstoned_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_checkpoints_project
                ON pipeline_checkpoints(project_id, phase_id);"
        )?;
        Ok(())
    }

    pub fn ensure_project(&self, id: &str, name: &str, root_path: &str) -> Result<(), GraphError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO projects (id, name, root_path) VALUES (?1, ?2, ?3)",
            params![id, name, root_path],
        )?;
        Ok(())
    }

    pub fn insert_node(&self, node: &Node) -> Result<i64, GraphError> {
        self.conn.execute(
            "INSERT INTO nodes (project_id, kind, label, file_path, line_start, line_end, properties_json, phase_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                node.project_id,
                node.kind,
                node.label,
                node.file_path,
                node.line_start,
                node.line_end,
                node.properties_json,
                node.phase_id,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_nodes_batch(&self, nodes: &[Node]) -> Result<Vec<i64>, GraphError> {
        let mut ids = Vec::with_capacity(nodes.len());
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO nodes (project_id, kind, label, file_path, line_start, line_end, properties_json, phase_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            )?;
            for node in nodes {
                stmt.execute(params![
                    node.project_id,
                    node.kind,
                    node.label,
                    node.file_path,
                    node.line_start,
                    node.line_end,
                    node.properties_json,
                    node.phase_id,
                ])?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit()?;
        Ok(ids)
    }

    pub fn insert_edge(&self, edge: &Edge) -> Result<i64, GraphError> {
        self.conn.execute(
            "INSERT INTO edges (project_id, source_id, target_id, kind, properties_json, phase_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                edge.project_id,
                edge.source_id,
                edge.target_id,
                edge.kind,
                edge.properties_json,
                edge.phase_id,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_edges_batch(&self, edges: &[Edge]) -> Result<Vec<i64>, GraphError> {
        let mut ids = Vec::with_capacity(edges.len());
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO edges (project_id, source_id, target_id, kind, properties_json, phase_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;
            for edge in edges {
                stmt.execute(params![
                    edge.project_id,
                    edge.source_id,
                    edge.target_id,
                    edge.kind,
                    edge.properties_json,
                    edge.phase_id,
                ])?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit()?;
        Ok(ids)
    }

    pub fn query_nodes_by_kind(&self, project_id: &str, kind: &str) -> Result<Vec<Node>, GraphError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, kind, label, file_path, line_start, line_end, properties_json, phase_id
             FROM nodes WHERE project_id = ?1 AND kind = ?2 AND tombstoned_at IS NULL"
        )?;
        let rows = stmt.query_map(params![project_id, kind], |row| {
            Ok(Node {
                id: Some(row.get(0)?),
                project_id: row.get(1)?,
                kind: row.get(2)?,
                label: row.get(3)?,
                file_path: row.get(4)?,
                line_start: row.get(5)?,
                line_end: row.get(6)?,
                properties_json: row.get(7)?,
                phase_id: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(GraphError::from)
    }

    pub fn query_nodes_by_file(&self, project_id: &str, file_path: &str) -> Result<Vec<Node>, GraphError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, kind, label, file_path, line_start, line_end, properties_json, phase_id
             FROM nodes WHERE project_id = ?1 AND file_path = ?2 AND tombstoned_at IS NULL"
        )?;
        let rows = stmt.query_map(params![project_id, file_path], |row| {
            Ok(Node {
                id: Some(row.get(0)?),
                project_id: row.get(1)?,
                kind: row.get(2)?,
                label: row.get(3)?,
                file_path: row.get(4)?,
                line_start: row.get(5)?,
                line_end: row.get(6)?,
                properties_json: row.get(7)?,
                phase_id: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(GraphError::from)
    }

    pub fn count_nodes_by_kind(&self, project_id: &str) -> Result<Vec<(String, i64)>, GraphError> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, COUNT(*) FROM nodes
             WHERE project_id = ?1 AND tombstoned_at IS NULL
             GROUP BY kind ORDER BY COUNT(*) DESC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(GraphError::from)
    }

    pub fn count_edges_by_kind(&self, project_id: &str) -> Result<Vec<(String, i64)>, GraphError> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, COUNT(*) FROM edges
             WHERE project_id = ?1 AND tombstoned_at IS NULL
             GROUP BY kind ORDER BY COUNT(*) DESC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(GraphError::from)
    }

    pub fn save_checkpoint(
        &self,
        project_id: &str,
        phase_id: u8,
        status: &str,
        started_at: &str,
        finished_at: Option<&str>,
        stats_json: Option<&str>,
    ) -> Result<(), GraphError> {
        self.conn.execute(
            "INSERT INTO pipeline_checkpoints (project_id, phase_id, status, started_at, finished_at, stats_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![project_id, phase_id, status, started_at, finished_at, stats_json],
        )?;
        Ok(())
    }

    pub fn get_last_completed_phase(&self, project_id: &str) -> Result<Option<u8>, GraphError> {
        let result = self.conn.query_row(
            "SELECT MAX(phase_id) FROM pipeline_checkpoints
             WHERE project_id = ?1 AND status = 'completed'",
            params![project_id],
            |row| row.get::<_, Option<u8>>(0),
        )?;
        Ok(result)
    }

    pub fn find_node_id_by_label(&self, project_id: &str, kind: &str, label: &str) -> Result<Option<i64>, GraphError> {
        let result = self.conn.query_row(
            "SELECT id FROM nodes
             WHERE project_id = ?1 AND kind = ?2 AND label = ?3 AND tombstoned_at IS NULL
             LIMIT 1",
            params![project_id, kind, label],
            |row| row.get::<_, i64>(0),
        );
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(GraphError::from(e)),
        }
    }

    pub fn update_node_properties(&self, node_id: i64, properties_json: &str) -> Result<(), GraphError> {
        self.conn.execute(
            "UPDATE nodes SET properties_json = ?1 WHERE id = ?2",
            params![properties_json, node_id],
        )?;
        Ok(())
    }
}
