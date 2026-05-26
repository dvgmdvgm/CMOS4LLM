# ADR-012: SQLite with recursive CTEs as the graph store for L4

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-002, ADR-009, ADR-011

## Context

L4 (Project memory) requires a graph store for: code symbol relationships (calls, imports, inheritance), domain ontology (entities, FK edges), decision graphs (ADRs → policies → evidence), and module dependencies. The store must be embedded (no separate server), support the append-only/tombstone model (ADR-009), and run on Windows.

Dedicated embedded graph DBs evaluated:
- **Kuzu** — project archived (2025). Dead.
- **CozoDB** — last release Dec 2023. Effectively abandoned.
- **IndraDB** — native Rust, but low adoption (~1.2K downloads/month), no query language (programmatic API only).
- **Oxigraph** — RDF/SPARQL model, wrong paradigm for property graphs.

## Decision

**L4 graph storage uses SQLite (via rusqlite) with a relational schema modeling nodes, edges, and properties. Graph traversal uses recursive CTEs.** No dedicated graph database.

## Rationale

1. **No viable embedded graph DB exists in the Rust ecosystem (2026).** The two best candidates (Kuzu, CozoDB) are dead or abandoned. Building on a dead dependency is worse than building on a boring one.
2. **SQLite is the most battle-tested embedded database in existence** — 7.5M downloads/month for rusqlite, used in 4846 crates. Zero risk of abandonment.
3. **Recursive CTEs handle graph traversal** — `WITH RECURSIVE` supports BFS/DFS over adjacency lists. For CMOS's graph sizes (10K–100K nodes for a 400K LoC project), this is performant enough (<100ms for 5-hop traversals).
4. **SQL is universally understood** — no Datalog or Cypher learning curve. Debugging and ad-hoc queries are trivial.
5. **Unified storage engine** — L2/L3 event log also uses SQLite (ADR-016). One engine to tune, backup, and understand.
6. **Append-only model maps naturally** — nodes/edges have `created_at`, `tombstoned_at` columns. Version chains via `supersedes` FK. Time-travel queries are just `WHERE created_at <= @t AND (tombstoned_at IS NULL OR tombstoned_at > @t)`.

## Consequences

### Positive
- Zero additional dependencies beyond rusqlite (already needed for L2/L3).
- Proven reliability, crash safety (WAL mode), and Windows support.
- Full SQL expressiveness for complex queries (JOINs, aggregations, window functions).
- Easy to inspect and debug with any SQLite client.
- Schema migrations via standard tools (refinery, sqlx).

### Negative
- No native graph query language — complex traversals require hand-written recursive CTEs.
- Performance ceiling for very deep traversals (>10 hops) or very large graphs (>1M nodes). Unlikely for desktop use but possible in V2+ multi-project scenarios.
- No built-in graph algorithms (PageRank, community detection) — must implement in application code or use a library.

### Neutral / unknowns
- Whether FTS5 (SQLite full-text search) is sufficient for text search within graph nodes, or whether LanceDB handles that.
- Exact schema design (adjacency list vs edge table vs both) — to be determined during M1 implementation.

## Alternatives considered

- **IndraDB:** Native Rust, RocksDB backend. Rejected: low adoption, no query language (all traversals are programmatic), unclear maintenance trajectory. If we're writing traversal code anyway, SQLite gives us SQL for free.
- **Custom graph layer on redb:** Pure Rust, no C deps. Rejected: redb is key-value only — we'd be building an entire query engine from scratch. SQLite already has one.
- **Neo4j/Memgraph (server mode):** Rejected: requires separate server process, violates embedded constraint, deployment complexity for desktop app.
- **Keep searching for a graph DB:** Rejected: the Rust embedded graph ecosystem is barren in 2026. Waiting for one to mature delays MVP indefinitely. SQLite is good enough now and can be swapped later if a viable option emerges.

## Implementation notes

- Schema: `nodes(id, project_id, kind, label, properties_json, created_at, tombstoned_at, supersedes)`, `edges(id, project_id, source_id, target_id, kind, properties_json, created_at, tombstoned_at)`.
- Indexes: `(project_id, kind)`, `(source_id, kind)`, `(target_id, kind)` for fast traversal.
- Recursive CTE helper functions in a `graph` module wrapping common patterns (ancestors, descendants, shortest path, subgraph extraction).
- Same SQLite file as L2/L3 or separate file — TBD based on write contention patterns. Likely separate file for L4 (different access patterns).

## Revisit conditions

- If a maintained, embedded, Rust-native property graph DB with a query language reaches >10K downloads/month and stable API — evaluate migration.
- If L4 graph exceeds 1M nodes and recursive CTE performance becomes a bottleneck — consider migrating hot subgraphs to an in-memory adjacency structure with SQLite as persistence backend.
