# ADR-016: SQLite WAL as the storage backend for L2/L3 event log

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-002, ADR-005, ADR-011, ADR-012

## Context

L2 (Session memory) and L3 (Episodic memory) are event-sourced stores. L2 records every turn, decision, and scratch fact within a session as immutable events. L3 stores completed episodes (task summaries, lessons, rejected approaches). Both require: append-heavy writes, fast sequential reads for replay, temporal queries for time-travel debugging (ADR-005), and crash safety on Windows.

ADR-012 already chose SQLite for the L4 graph store. Using the same engine for L2/L3 reduces operational complexity.

## Decision

**L2 and L3 use SQLite in WAL (Write-Ahead Logging) mode via the `rusqlite` crate.** Each project gets its own SQLite database file. Events are stored as immutable rows with structured metadata columns and a JSON payload column.

## Rationale

1. **SQL for time-travel queries:** `SELECT * FROM events WHERE project_id = ? AND timestamp BETWEEN ? AND ? ORDER BY timestamp` — trivial to write, trivial to debug. No custom query engine needed.
2. **WAL mode:** Concurrent readers don't block writers. Critical for the live inference path (writing events) while the GUI reads historical data simultaneously.
3. **Crash safety:** SQLite's WAL + checkpointing guarantees durability even on power loss. Essential for a long-running daemon managing persistent state.
4. **Zero build complexity on Windows:** `rusqlite` with `bundled` feature compiles SQLite from source — no system dependencies, no clang required (unlike RocksDB).
5. **Unified engine:** L2, L3, and L4 all use SQLite (ADR-012). One engine to understand, tune, backup, and migrate. Reduces cognitive load and dependency count.
6. **7.5M downloads/month:** Most battle-tested embedded database. Zero risk of abandonment or obscure bugs.
7. **Rich query capabilities:** Window functions, CTEs, JSON functions, FTS5 — all available for complex analytics (token usage trends, episode similarity, drift patterns) without additional dependencies.

## Consequences

### Positive
- Single dependency (rusqlite) covers L2, L3, and L4 storage needs.
- SQL makes ad-hoc debugging and data inspection trivial (any SQLite client works).
- WAL mode provides excellent concurrent read/write performance for desktop workloads.
- Mature tooling for schema migrations (refinery crate or manual versioned scripts).
- Backup is just file copy (with proper checkpoint).

### Negative
- Not optimized for pure append workloads the way LSM-tree (RocksDB) is — but at CMOS's event volume (hundreds/day, not millions/second), this is irrelevant.
- No built-in compression — events stored at full size. Mitigated by JSON payload (compressible) and periodic archival to L5.
- Single-writer limitation in WAL mode — only one connection can write at a time. Acceptable for a single-daemon architecture.
- Row-based storage is less efficient than columnar for analytical scans over many events — but again, at desktop scale this doesn't matter.

### Neutral / unknowns
- Whether L2, L3, and L4 share one SQLite file or use separate files per layer — separate files likely better (different access patterns, independent vacuuming).
- Exact event schema (columns vs pure JSON) — TBD during M2 implementation. Likely: structured columns for indexed fields (timestamp, project_id, event_type, entity_id) + JSON blob for payload.
- Whether FTS5 is needed for full-text search within events or whether LanceDB handles all semantic search.

## Alternatives considered

- **RocksDB:** LSM-tree, excellent for append-heavy workloads at scale. Rejected: (a) requires clang/LLVM on Windows — significant build complexity; (b) key-value only — would need to build query logic for time-travel, temporal ranges, and analytics; (c) overkill for desktop event volumes; (d) last Rust crate release Aug 2025 — may lag upstream.
- **redb:** Pure Rust, ACID, crash-safe, zero C dependencies. Rejected: (a) key-value only — same query logic problem as RocksDB; (b) no SQL means time-travel queries require custom code; (c) less mature than SQLite (though actively maintained). Would be the choice if we needed pure-Rust with no C deps, but SQLite's query capabilities outweigh that benefit.
- **Separate event store (EventStoreDB, NATS JetStream):** Rejected: requires separate server process, violates embedded constraint, deployment complexity for desktop app.

## Implementation notes

- Crate: `rusqlite` with `bundled` feature (compiles SQLite from source).
- Database files: `~/.cmos/projects/<project_id>/events.db` (L2/L3), separate from `graph.db` (L4).
- WAL mode enabled on connection open: `PRAGMA journal_mode=WAL;`
- Key pragmas: `PRAGMA synchronous=NORMAL;` (safe with WAL), `PRAGMA foreign_keys=ON;`
- Schema (draft):
  ```sql
  CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL,
    layer TEXT NOT NULL CHECK(layer IN ('L2', 'L3')),
    event_type TEXT NOT NULL,
    entity_id TEXT,
    timestamp TEXT NOT NULL, -- ISO 8601
    payload TEXT NOT NULL,   -- JSON
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
  );
  CREATE INDEX idx_events_project_time ON events(project_id, timestamp);
  CREATE INDEX idx_events_entity ON events(project_id, entity_id) WHERE entity_id IS NOT NULL;
  ```
- Connection pool: `r2d2-sqlite` or manual pool with read-only connections for queries + single write connection.
- Archival: periodic job moves old L2 events to L5 (object storage) after session closes.

## Revisit conditions

- If event volume exceeds SQLite's comfortable write throughput (unlikely on desktop — SQLite handles ~50K inserts/sec in WAL mode).
- If we need true columnar analytics over millions of events — consider DuckDB (also embeddable, SQL, columnar) as a read-optimized companion.
- If pure-Rust requirement becomes important (e.g., for WASM compilation) — migrate to redb + custom query layer.
