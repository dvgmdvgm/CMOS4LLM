use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct TokenStats {
    pub total_requests: u64,
    pub tokens_saved: u64,
    pub tokens_without_cmos: u64,
    pub tokens_with_cmos: u64,
    pub savings_ratio: f64,
}

#[derive(Clone)]
pub struct TokenTracker {
    inner: Arc<TokenTrackerInner>,
}

struct TokenTrackerInner {
    total_requests: AtomicU64,
    tokens_with_cmos: AtomicU64,
    tokens_without_cmos: AtomicU64,
    db_path: Option<std::path::PathBuf>,
}

impl TokenTracker {
    pub fn new(data_root: &Path) -> Self {
        let db_path = data_root.join("token_analytics.db");
        let tracker = Self {
            inner: Arc::new(TokenTrackerInner {
                total_requests: AtomicU64::new(0),
                tokens_with_cmos: AtomicU64::new(0),
                tokens_without_cmos: AtomicU64::new(0),
                db_path: Some(db_path.clone()),
            }),
        };

        if let Ok(conn) = Self::open_db(&db_path) {
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS token_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                    project_id TEXT NOT NULL,
                    tool_name TEXT NOT NULL,
                    tokens_assembled INTEGER NOT NULL,
                    tokens_baseline_estimate INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_token_events_project
                    ON token_events(project_id, timestamp);"
            );

            if let Ok(row) = conn.query_row(
                "SELECT COUNT(*), COALESCE(SUM(tokens_assembled), 0), COALESCE(SUM(tokens_baseline_estimate), 0) FROM token_events",
                [],
                |row| Ok((row.get::<_, u64>(0)?, row.get::<_, u64>(1)?, row.get::<_, u64>(2)?)),
            ) {
                tracker.inner.total_requests.store(row.0, Ordering::Relaxed);
                tracker.inner.tokens_with_cmos.store(row.1, Ordering::Relaxed);
                tracker.inner.tokens_without_cmos.store(row.2, Ordering::Relaxed);
            }
        }

        tracker
    }

    pub fn in_memory() -> Self {
        Self {
            inner: Arc::new(TokenTrackerInner {
                total_requests: AtomicU64::new(0),
                tokens_with_cmos: AtomicU64::new(0),
                tokens_without_cmos: AtomicU64::new(0),
                db_path: None,
            }),
        }
    }

    pub fn record(&self, project_id: &str, tool_name: &str, tokens_assembled: u64, tokens_baseline_estimate: u64) {
        self.inner.total_requests.fetch_add(1, Ordering::Relaxed);
        self.inner.tokens_with_cmos.fetch_add(tokens_assembled, Ordering::Relaxed);
        self.inner.tokens_without_cmos.fetch_add(tokens_baseline_estimate, Ordering::Relaxed);

        if let Some(ref db_path) = self.inner.db_path
            && let Ok(conn) = Self::open_db(db_path)
        {
            let _ = conn.execute(
                "INSERT INTO token_events (project_id, tool_name, tokens_assembled, tokens_baseline_estimate) VALUES (?1, ?2, ?3, ?4)",
                params![project_id, tool_name, tokens_assembled, tokens_baseline_estimate],
            );
        }
    }

    pub fn stats(&self) -> TokenStats {
        let total_requests = self.inner.total_requests.load(Ordering::Relaxed);
        let tokens_with_cmos = self.inner.tokens_with_cmos.load(Ordering::Relaxed);
        let tokens_without_cmos = self.inner.tokens_without_cmos.load(Ordering::Relaxed);
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

    fn open_db(path: &Path) -> Result<Connection, rusqlite::Error> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(conn)
    }
}
