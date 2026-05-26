# ADR-013: LanceDB as the vector index for L3/L4

- **Status:** Superseded by ADR-017
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-002, ADR-011, ADR-012

## Context

CMOS needs vector similarity search for: episodic retrieval (L3 — "find similar past tasks"), semantic search over project knowledge (L4 — "find relevant code/docs for this query"), and embedding-based deduplication. Requirements: embedded (in-process), Rust-native or excellent Rust bindings, Windows support, persistence, and ideally hybrid search (vector + metadata filtering).

## Decision

**LanceDB is the vector index for L3 and L4 semantic search.** It runs embedded (local filesystem mode), provides IVF-PQ indexing, metadata filtering, full-text search, and versioned storage — all via native Rust API.

## Rationale

1. **Batteries-included:** LanceDB is not just a vector index — it's a vector database with persistence, versioning, SQL-like filtering, and full-text search. This eliminates glue code for metadata filtering and persistence that USearch would require.
2. **Native Rust:** Built on Apache Arrow and Lance columnar format. 109K downloads/month, very active development (v0.29.0, May 2026).
3. **Versioning built-in:** Lance format supports time-travel natively (append-only, version manifests). Aligns perfectly with ADR-009 (append-only memory) and ADR-005 (time travel debugging).
4. **Hybrid search:** Vector similarity + full-text search + metadata filters in a single query. Critical for Retrieval Router which combines multiple strategies.
5. **Windows support:** Local filesystem mode works on all platforms.
6. **Zero-copy Arrow integration:** Efficient for batch operations during bootstrap (M1) and background consolidation.

## Consequences

### Positive
- Single dependency covers vector search, full-text search, metadata filtering, and persistence.
- Built-in versioning aligns with append-only architecture.
- No separate server process — embedded in the Rust daemon.
- Scales to millions of vectors with IVF-PQ quantization.
- Active development with frequent releases.

### Negative
- Heavy dependency tree (Arrow, DataFusion, Lance ecosystem) — increases compile time and binary size.
- Relatively young project — API may have breaking changes between minor versions.
- IVF-PQ requires a training step (index build) — cold start after bootstrap takes time.
- Overkill if we only needed simple kNN on <10K vectors (but we'll grow beyond that).

### Neutral / unknowns
- Embedding model choice (which model generates the vectors) is orthogonal to this decision — handled by Sub-LM runtime.
- Whether L3 and L4 share one LanceDB instance or separate — TBD based on access patterns.
- Interaction with SQLite graph store — likely: SQLite stores structure/relationships, LanceDB stores embeddings. Cross-reference via shared IDs.

## Alternatives considered

- **USearch:** Ultra-fast HNSW, lightweight, 87K downloads/month. Rejected: it's only an index — no persistence layer, no metadata filtering, no full-text search. We'd need to build all of that ourselves on top. The performance advantage (10x over FAISS) is irrelevant at CMOS's scale (<1M vectors on desktop).
- **Qdrant embedded/Edge:** Full-featured vector DB. Rejected: primarily a server product, Edge mode on Windows is poorly documented, heavier than LanceDB, potential licensing concerns for embedding.
- **hnswlib:** No maintained Rust crate found. Effectively superseded by USearch.
- **FAISS:** Python-first, C++ core. Poor Rust integration, no embedded persistence.

## Implementation notes

- Crate: `lancedb` (Rust SDK).
- Storage: local filesystem under CMOS data directory (e.g., `~/.cmos/projects/<id>/vectors/`).
- Tables: `episodic_embeddings` (L3), `knowledge_embeddings` (L4), `code_embeddings` (L4 symbols).
- Embedding dimensions: TBD based on Sub-LM embedding model (likely 384–1024d).
- Index type: IVF-PQ for tables >10K rows, flat for smaller tables.
- Integration: shared node/edge IDs between SQLite graph and LanceDB vectors — SQLite is the structural source of truth, LanceDB provides semantic similarity.

## Revisit conditions

- If LanceDB development stalls or the project is abandoned — evaluate USearch + custom persistence layer.
- If binary size becomes a deployment concern (LanceDB + Arrow adds significant weight) — consider stripping unused Arrow features or switching to USearch for MVP with LanceDB deferred to V1.
- If embedding dimensions or vector count exceed LanceDB's comfortable range on desktop hardware — unlikely but would require sharding or a different index strategy.
