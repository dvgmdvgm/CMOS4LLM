# ADR-002: Five-layer memory hierarchy (L1–L5)

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-001 (stateless LLM), ADR-003 (two-LLM economy)

## Context

Most memory systems collapse all retention into one or two stores: a vector DB plus a chat log, or a knowledge graph plus a document index. This works for toy use cases but fails the moment you need:

- Different latency budgets for "what was just said" vs. "what we decided last March."
- Different durability for transient scratch vs. project constitution.
- Different access patterns: append-only stream (sessions) vs. random-access graph (project ontology) vs. cold archive.
- Different conflict-resolution rules per layer.

Stuffing it all into one store either makes hot reads slow or makes cold storage expensive and lossy.

## Decision

CMOS uses a strict 5-layer memory hierarchy with explicit per-layer properties:

| Layer | Size | TTL | Latency | Tech | Contents |
|---|---|---|---|---|---|
| **L1 Working** | 1K–16K tokens | minutes | <1ms | RAM, lock-free | currently-assembled prompt + scratch |
| **L2 Session** | 50K–500K tokens | hours | <5ms | RocksDB + event log | current session: turns, decisions, scratch facts |
| **L3 Episodic** | 1M–10M tokens | days–weeks | <50ms | RocksDB + vector | tasks, lessons, rejected approaches |
| **L4 Project** | 100M–10B tokens | indefinite | <100ms | Graph DB + vector + KV | symbol graph, ontology, policies, DNA |
| **L5 Archival** | unbounded | indefinite (decay) | <1s | object store + cold KG | full history, evolution, deprecated |

Promotion (L → L+1) and demotion (L → L+1 cold) are automatic, governed by access patterns + semantic importance + recency. Hard-delete from L4/L5 is forbidden; only tombstones with version chains.

## Rationale

- **Each layer's tech matches its access pattern.** L1 must be in-process (zero IPC overhead); L4 is a graph because relationships dominate; L5 is object storage because append-rare-read.
- **Latency budgets are the constraint that drives the split.** A single store that's good at <1ms and at <1s simultaneously does not exist.
- **Promotion model embeds the value judgment.** Frequent access + semantic importance is a learnable signal; it's exactly what determines whether a fact deserves to live forever or expire.
- **Append-only + tombstones preserve evolution.** A project's history is not edits — it's a series of decisions, some of which supersede others. Throwing away the old breaks the ability to answer "why is the code shaped this way?"
- **Conflict resolution is layer-aware.** A conflict in L2 (within a session) may auto-resolve by recency. A conflict in L4 (between two project-level invariants) must surface to the human.

## Consequences

### Positive
- Hot path (L1+L2) stays fast even with terabyte L5.
- L4 is "the thing that survives forever" — clear ownership.
- Time Travel Debugging is natural: replay any past inference by reading L2 events for that turn + L4 state at that moment.
- Multi-project isolation is just multi-tenant L2/L3/L4/L5 namespaces.

### Negative
- Five storage backends to operate, not one.
- Promotion/demotion logic is non-trivial and must be carefully tuned.
- Cross-layer queries (e.g., "all references to entity X across all layers") need a layer-aware retrieval router.

### Neutral / unknowns
- Optimal promotion thresholds (access count, recency window, semantic-importance scoring) must be learned empirically per project.
- Whether L3 should split further (active episodes vs. completed) — TBD after MVP usage.

## Alternatives considered

- **Single store (one big graph DB):** rejected — latency for L1-class hot reads becomes graph-traversal-bound; embedding hot working-state in a remote graph is wasteful.
- **Two layers (hot + cold):** considered. Insufficient: episodic memory has a distinct shape (tasks with rejected approaches and lessons) that doesn't fit "hot session" or "cold archive."
- **Per-layer separate products (Mem0 + GraphRAG + ChromaDB + …):** rejected — gluing four open-source systems means four upgrade cycles, four schema migrations, and four observability stories. Composition is fine; choosing four heterogeneous products is not.

## Implementation notes

- L1 is always in-process inside Hypervisor (no IPC).
- L2 event log uses RocksDB or SQLite WAL; events are immutable, snapshots are derived.
- L3 needs both fast lookup-by-ID and similar-task vector retrieval; a single store with both indexes (e.g., RocksDB + hnswlib sidecar, or LanceDB) is the goal.
- L4 is composite: pick a graph DB (ADR-012), a vector index (ADR-013), and a KV store. Keep them in the same process to avoid distributed transactions.
- L5 is object storage (S3-compat or local FS) + parquet/lance + cold KG snapshots.
- Tombstones carry: timestamp, reason, supersedes_id (if applicable), evidence_refs.

## Revisit conditions

- If L3 turns out to be unused in practice (everything either short-lived in L2 or important in L4), collapse into 4 layers.
- If a single embedded DB emerges that satisfies all of {graph, vector, KV, time-travel} at L4 latencies, reconsider the composite L4 design.
- If projects grow beyond 10B-token L4, L4 itself may need internal sharding — out of scope for V1.
