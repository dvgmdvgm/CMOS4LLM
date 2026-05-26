# Component: Memory Layers (L1–L5)

> Five-layer memory hierarchy with strict per-layer properties ([ADR-002](../02-decisions/ADR-002-five-layer-memory.md)).

## Layer Summary

| Layer | Store | Latency | Mutability |
|-------|-------|---------|------------|
| L1 Working | RAM | <1ms | Ephemeral |
| L2 Session | RocksDB | <5ms | Append-only events |
| L3 Episodic | RocksDB + vector | <50ms | Append-only episodes |
| L4 Project | Graph + vector + KV | <100ms | Append-only, tombstones ([ADR-009](../02-decisions/ADR-009-append-only-memory-with-tombstones.md)) |
| L5 Archival | Object store | <1s | Append-only |

## Promotion & Demotion

TODO: Define promotion triggers (L2→L3, L3→L4)
TODO: Define demotion/decay rules (L3→L5)
TODO: Define Sub-LM importance scoring for promotion

## Conflict Resolution

TODO: Define version chain structure
TODO: Define auto-resolve vs ask-user threshold

## Scope

- [MVP M2](../03-scope/mvp.md) — L1–L4 functional, L5 log-only
- [V1.B](../03-scope/v1.md) — full L5 with retrieval

## Open Questions

TODO: Storage backend choices (ADR-012, ADR-013, ADR-016 pending)
TODO: Compaction strategy for L2 event log
