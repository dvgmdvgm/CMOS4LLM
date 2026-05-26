# ADR-017: USearch + SQLite replaces LanceDB for vector index

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Supersedes:** ADR-013
- **Related:** ADR-002, ADR-011, ADR-012, ADR-016

## Context

ADR-013 selected LanceDB as the vector index. During implementation, LanceDB proved impractical for this project:

1. **Requires `protoc` (Protocol Buffers compiler)** — not installed on owner's Windows machine, adds an external build dependency.
2. **Massive dependency tree** — pulls in DataFusion, Arrow, Lance, protobuf (~250+ crates). Compile time increased dramatically.
3. **Overkill for MVP scale** — CMOS on desktop will have <100K vectors. LanceDB's IVF-PQ, versioned storage, and SQL-like filtering are unnecessary at this scale.

The original rejection of USearch ("no persistence, no metadata filtering") is solvable with minimal code: SQLite stores metadata, USearch stores the HNSW index. Both are already dependencies in the project.

## Decision

**USearch (HNSW) + SQLite metadata table replaces LanceDB as the vector index.**

- USearch provides in-memory HNSW index with cosine similarity, serializable to disk.
- A SQLite table (`vector_meta`) stores the mapping: `key → (id, source_id, layer, content)`.
- Layer filtering is done post-search by checking metadata (with over-fetch to compensate).

## Rationale

1. **Zero external build deps:** USearch compiles via CXX bridge, no protoc needed. SQLite is already bundled via `rusqlite`.
2. **Lightweight:** USearch adds ~10 crates (CXX ecosystem). Compare to LanceDB's ~250.
3. **Fast compile:** Full workspace check in ~35s vs 90s+ with LanceDB.
4. **Sufficient for MVP:** HNSW gives excellent recall at <100K vectors. No training step needed (unlike IVF-PQ).
5. **Persistence is trivial:** `index.save()` / `index.load()` for the HNSW graph. SQLite handles metadata durably.
6. **Already proven:** All 16 retrieval tests pass, including vector search with layer filtering and upsert.

## Consequences

### Positive
- No external toolchain requirements (protoc, cmake, etc.).
- Compile time stays manageable on desktop hardware.
- Simple, auditable code — the full VectorIndex implementation is ~280 lines.
- SQLite metadata allows arbitrary filtering without USearch needing to know about it.

### Negative
- No built-in full-text search (LanceDB had this). Keyword search remains structural (query by kind/layer/session).
- No built-in versioning of the vector index (LanceDB's Lance format had time-travel). Acceptable: we version at the event/fact level (ADR-009), not at the index level.
- Post-search filtering is less efficient than pre-search filtering at scale. Acceptable at <100K vectors.
- USearch index file is not human-readable (binary HNSW graph).

### Neutral
- Embedding generation unchanged — still via Ollama `/api/embed`.
- Hybrid retrieval strategy unchanged — vector similarity (60%) + keyword scoring (40%).
- If we outgrow USearch (>1M vectors, need pre-filtering), we can revisit LanceDB or Qdrant in V2.

## Alternatives reconsidered

- **LanceDB:** Original choice (ADR-013). Rejected due to protoc requirement and dependency weight.
- **hora:** Pure Rust ANN library. Less mature, fewer downloads, no persistence API.
- **instant-distance:** Pure Rust HNSW. Simpler but less feature-rich than USearch, smaller community.

## Revisit conditions

- If vector count exceeds 1M and post-search filtering becomes a bottleneck.
- If full-text search over memory content becomes a requirement (would need a dedicated FTS solution).
- If USearch crate becomes unmaintained.
