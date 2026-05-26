use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection as SqliteConn};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

use crate::error::RetrievalError;

pub struct VectorIndex {
    index: Index,
    meta_db: Mutex<SqliteConn>,
    dimension: usize,
    next_key: Mutex<u64>,
}

#[derive(Debug, Clone)]
pub struct VectorRecord {
    pub id: String,
    pub source_id: i64,
    pub layer: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: String,
    pub source_id: i64,
    pub layer: String,
    pub content: String,
    pub distance: f32,
}

impl VectorIndex {
    pub fn open(path: &Path, dimension: usize) -> Result<Self, RetrievalError> {
        let meta_path = path.join("vector_meta.db");
        let index_path = path.join("vector.usearch");

        std::fs::create_dir_all(path)
            .map_err(|e| RetrievalError::VectorIndex(format!("cannot create dir: {}", e)))?;

        let meta_db = SqliteConn::open(&meta_path)
            .map_err(|e| RetrievalError::VectorIndex(format!("sqlite open failed: {}", e)))?;

        meta_db
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS vector_meta (
                    key INTEGER PRIMARY KEY,
                    id TEXT NOT NULL UNIQUE,
                    source_id INTEGER NOT NULL,
                    layer TEXT NOT NULL,
                    content TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_meta_layer ON vector_meta(layer);
                CREATE INDEX IF NOT EXISTS idx_meta_source ON vector_meta(source_id);",
            )
            .map_err(|e| RetrievalError::VectorIndex(format!("schema init failed: {}", e)))?;

        let next_key: u64 = meta_db
            .query_row("SELECT COALESCE(MAX(key), 0) + 1 FROM vector_meta", [], |r| {
                r.get(0)
            })
            .unwrap_or(1);

        let options = IndexOptions {
            dimensions: dimension,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
            ..Default::default()
        };

        let index = Index::new(&options)
            .map_err(|e| RetrievalError::VectorIndex(format!("index create failed: {}", e)))?;

        if index_path.exists() {
            index
                .load(index_path.to_str().unwrap_or(""))
                .map_err(|e| RetrievalError::VectorIndex(format!("index load failed: {}", e)))?;
        } else {
            index
                .reserve(1024)
                .map_err(|e| RetrievalError::VectorIndex(format!("index reserve failed: {}", e)))?;
        }

        Ok(Self {
            index,
            meta_db: Mutex::new(meta_db),
            dimension,
            next_key: Mutex::new(next_key),
        })
    }

    pub fn open_in_memory(dimension: usize) -> Result<Self, RetrievalError> {
        let meta_db = SqliteConn::open_in_memory()
            .map_err(|e| RetrievalError::VectorIndex(format!("sqlite open failed: {}", e)))?;

        meta_db
            .execute_batch(
                "CREATE TABLE vector_meta (
                    key INTEGER PRIMARY KEY,
                    id TEXT NOT NULL UNIQUE,
                    source_id INTEGER NOT NULL,
                    layer TEXT NOT NULL,
                    content TEXT NOT NULL
                );
                CREATE INDEX idx_meta_layer ON vector_meta(layer);
                CREATE INDEX idx_meta_source ON vector_meta(source_id);",
            )
            .map_err(|e| RetrievalError::VectorIndex(format!("schema init failed: {}", e)))?;

        let options = IndexOptions {
            dimensions: dimension,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
            ..Default::default()
        };

        let index = Index::new(&options)
            .map_err(|e| RetrievalError::VectorIndex(format!("index create failed: {}", e)))?;

        index
            .reserve(1024)
            .map_err(|e| RetrievalError::VectorIndex(format!("index reserve failed: {}", e)))?;

        Ok(Self {
            index,
            meta_db: Mutex::new(meta_db),
            dimension,
            next_key: Mutex::new(1),
        })
    }

    pub fn upsert(&self, records: &[VectorRecord]) -> Result<(), RetrievalError> {
        if records.is_empty() {
            return Ok(());
        }

        let mut next_key = self.next_key.lock().unwrap();
        let db = self.meta_db.lock().unwrap();

        let current_capacity = self.index.capacity();
        let needed = *next_key as usize + records.len();
        if needed > current_capacity {
            self.index
                .reserve(needed.max(current_capacity * 2))
                .map_err(|e| RetrievalError::VectorIndex(format!("reserve failed: {}", e)))?;
        }

        let tx = db
            .unchecked_transaction()
            .map_err(|e| RetrievalError::VectorIndex(format!("transaction failed: {}", e)))?;

        for record in records {
            if record.embedding.len() != self.dimension {
                return Err(RetrievalError::VectorIndex(format!(
                    "dimension mismatch: expected {}, got {}",
                    self.dimension,
                    record.embedding.len()
                )));
            }

            let existing_key: Option<u64> = tx
                .query_row(
                    "SELECT key FROM vector_meta WHERE id = ?1",
                    params![record.id],
                    |r| r.get(0),
                )
                .ok();

            let key = if let Some(k) = existing_key {
                self.index
                    .remove(k)
                    .map_err(|e| RetrievalError::VectorIndex(format!("remove failed: {}", e)))?;
                tx.execute(
                    "UPDATE vector_meta SET source_id=?1, layer=?2, content=?3 WHERE key=?4",
                    params![record.source_id, record.layer, record.content, k],
                )
                .map_err(|e| RetrievalError::VectorIndex(format!("update failed: {}", e)))?;
                k
            } else {
                let k = *next_key;
                *next_key += 1;
                tx.execute(
                    "INSERT INTO vector_meta (key, id, source_id, layer, content) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![k, record.id, record.source_id, record.layer, record.content],
                )
                .map_err(|e| RetrievalError::VectorIndex(format!("insert failed: {}", e)))?;
                k
            };

            self.index
                .add(key, &record.embedding)
                .map_err(|e| RetrievalError::VectorIndex(format!("index add failed: {}", e)))?;
        }

        tx.commit()
            .map_err(|e| RetrievalError::VectorIndex(format!("commit failed: {}", e)))?;

        Ok(())
    }

    pub fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
        layer_filter: Option<&str>,
    ) -> Result<Vec<VectorSearchResult>, RetrievalError> {
        if self.index.size() == 0 {
            return Ok(Vec::new());
        }

        let fetch_count = if layer_filter.is_some() {
            limit * 4
        } else {
            limit
        };

        let results = self
            .index
            .search(query_vector, fetch_count)
            .map_err(|e| RetrievalError::VectorIndex(format!("search failed: {}", e)))?;

        let db = self.meta_db.lock().unwrap();
        let mut search_results = Vec::with_capacity(limit);

        for (key, distance) in results.keys.iter().zip(results.distances.iter()) {
            let row = db.query_row(
                "SELECT id, source_id, layer, content FROM vector_meta WHERE key = ?1",
                params![key],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                },
            );

            let (id, source_id, layer, content) = match row {
                Ok(r) => r,
                Err(_) => continue,
            };

            if let Some(filter) = layer_filter
                && layer != filter
            {
                continue;
            }

            search_results.push(VectorSearchResult {
                id,
                source_id,
                layer,
                content,
                distance: *distance,
            });

            if search_results.len() >= limit {
                break;
            }
        }

        Ok(search_results)
    }

    pub fn count(&self) -> usize {
        self.index.size()
    }

    pub fn save(&self, path: &Path) -> Result<(), RetrievalError> {
        let index_path = path.join("vector.usearch");
        self.index
            .save(index_path.to_str().unwrap_or(""))
            .map_err(|e| RetrievalError::VectorIndex(format!("save failed: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(id: &str, source_id: i64, layer: &str, content: &str, dim: usize) -> VectorRecord {
        VectorRecord {
            id: id.to_string(),
            source_id,
            layer: layer.to_string(),
            content: content.to_string(),
            embedding: vec![0.1; dim],
        }
    }

    #[test]
    fn test_open_in_memory() {
        let index = VectorIndex::open_in_memory(768).unwrap();
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn test_upsert_and_count() {
        let index = VectorIndex::open_in_memory(8).unwrap();
        let records = vec![
            make_record("r1", 1, "L3", "first episode", 8),
            make_record("r2", 2, "L4", "a decision fact", 8),
        ];
        index.upsert(&records).unwrap();
        assert_eq!(index.count(), 2);
    }

    #[test]
    fn test_search_basic() {
        let index = VectorIndex::open_in_memory(4).unwrap();
        let records = vec![
            VectorRecord {
                id: "a".into(),
                source_id: 1,
                layer: "L3".into(),
                content: "episode about auth".into(),
                embedding: vec![1.0, 0.0, 0.0, 0.0],
            },
            VectorRecord {
                id: "b".into(),
                source_id: 2,
                layer: "L4".into(),
                content: "decision about database".into(),
                embedding: vec![0.0, 1.0, 0.0, 0.0],
            },
            VectorRecord {
                id: "c".into(),
                source_id: 3,
                layer: "L3".into(),
                content: "episode about payments".into(),
                embedding: vec![0.0, 0.0, 1.0, 0.0],
            },
        ];
        index.upsert(&records).unwrap();

        let results = index.search(&[1.0, 0.1, 0.0, 0.0], 2, None).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn test_search_with_layer_filter() {
        let index = VectorIndex::open_in_memory(4).unwrap();
        let records = vec![
            VectorRecord {
                id: "a".into(),
                source_id: 1,
                layer: "L3".into(),
                content: "episode".into(),
                embedding: vec![1.0, 0.0, 0.0, 0.0],
            },
            VectorRecord {
                id: "b".into(),
                source_id: 2,
                layer: "L4".into(),
                content: "fact".into(),
                embedding: vec![1.0, 0.1, 0.0, 0.0],
            },
        ];
        index.upsert(&records).unwrap();

        let results = index.search(&[1.0, 0.0, 0.0, 0.0], 10, Some("L4")).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].layer, "L4");
    }

    #[test]
    fn test_upsert_updates_existing() {
        let index = VectorIndex::open_in_memory(4).unwrap();
        let records = vec![VectorRecord {
            id: "a".into(),
            source_id: 1,
            layer: "L3".into(),
            content: "original".into(),
            embedding: vec![1.0, 0.0, 0.0, 0.0],
        }];
        index.upsert(&records).unwrap();
        assert_eq!(index.count(), 1);

        let updated = vec![VectorRecord {
            id: "a".into(),
            source_id: 1,
            layer: "L3".into(),
            content: "updated".into(),
            embedding: vec![0.0, 1.0, 0.0, 0.0],
        }];
        index.upsert(&updated).unwrap();
        assert_eq!(index.count(), 1);

        let results = index.search(&[0.0, 1.0, 0.0, 0.0], 1, None).unwrap();
        assert_eq!(results[0].content, "updated");
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let index = VectorIndex::open_in_memory(4).unwrap();
        let records = vec![VectorRecord {
            id: "a".into(),
            source_id: 1,
            layer: "L3".into(),
            content: "test".into(),
            embedding: vec![1.0, 0.0],
        }];
        assert!(index.upsert(&records).is_err());
    }
}
