# 2026-05-26 — Vector Index: USearch + Hybrid Retrieval

> Semantic retrieval для L3/L4 через HNSW (USearch) + hybrid scoring.

---

## Контекст

M3 (Two-LLM economy) был завершён в предыдущей сессии. Retrieval crate имел только keyword/structural retrieval (query by kind, layer, session). Следующий шаг по roadmap — vector index для semantic similarity search.

---

## Что сделано

### `crates/retrieval/src/embedding.rs` (новый)
- `EmbeddingClient`: async клиент к Ollama `/api/embed`.
- Batch embedding, auto-detection dimension по модели (nomic-embed-text → 768, mxbai-embed-large → 1024).
- 2 unit теста.

### `crates/retrieval/src/vector.rs` (новый)
- `VectorIndex`: USearch HNSW (cosine) + SQLite metadata table.
- `upsert()`: insert or update с dimension validation.
- `search()`: ANN search с post-filtering по layer (over-fetch 4x для компенсации).
- `save()`/`open()`: persistence на диск.
- 5 unit тестов (in-memory, upsert, search, layer filter, update existing, dimension mismatch).

### `crates/retrieval/src/hybrid.rs` (новый)
- `HybridRetriever`: combines vector similarity (60%) + keyword scoring (40%).
- `retrieve_l4()`, `retrieve_l3()`: async methods, budget-aware truncation.
- `merge_l4_scores()`, `merge_l3_scores()`: score fusion logic.
- 3 unit теста.

### `crates/retrieval/src/assembly.rs` (изменён)
- Добавлен `assemble_hybrid()`: использует vector index когда доступен, fallback на keyword-only.

### `crates/cli/src/main.rs` (изменён)
- `cmos vector index --project X --root Y` — индексирует L3/L4 в vector store.
- `cmos vector search --project X --root Y --query "..."` — semantic search.
- `cmos vector stats --root Y` — статистика индекса.

### Workspace Cargo.toml (изменён)
- Добавлены workspace deps: `usearch`, `rusqlite`, `reqwest`, `async-trait`, `chrono`, `futures`.
- `crates/sub-lm/Cargo.toml` переведён на workspace deps.

### ADR-017 (новый)
- `docs/02-decisions/ADR-017-usearch-replaces-lancedb.md` — supersedes ADR-013.

---

## Ключевые решения

1. **Pivot от LanceDB к USearch** — LanceDB требует `protoc` (не установлен на Windows owner'а) и тянет ~250 crates (DataFusion, Arrow, Lance, protobuf). USearch — lightweight HNSW через CXX bridge, ~10 crates, zero external build deps. Для <100K vectors на desktop HNSW более чем достаточен.

2. **SQLite metadata рядом с USearch** — USearch хранит только `(key: u64, embedding)`. Metadata (id, source_id, layer, content) живёт в SQLite таблице `vector_meta`. Это позволяет layer filtering без модификации HNSW индекса.

3. **Post-search filtering** — вместо pre-filtering (как в LanceDB) делаем over-fetch (4x limit) и фильтруем после. При <100K vectors это negligible overhead.

4. **Hybrid scoring 60/40** — vector similarity получает больший вес (0.6) чем keyword scoring (0.4). Rationale: semantic similarity лучше ловит intent, keyword scoring добавляет recency/importance bias.

5. **`assemble_hybrid` как отдельный метод** — не ломает существующий `assemble()`. Caller решает, передавать ли vector_index + embedding_client. Graceful degradation.

---

## Открытые вопросы

- Retrieval quality: нет synthetic benchmark'а для precision/recall. Нужен тестовый корпус.
- Embedding model choice: nomic-embed-text vs mxbai-embed-large — нужен A/B на реальных данных.
- Auto-indexing: сейчас `cmos vector index` — ручная команда. В будущем — background indexing при каждом write в L3/L4.

---

## Следующий шаг

MCP Server (ADR-010). См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы созданы:** `crates/retrieval/src/{embedding,vector,hybrid}.rs`, `docs/02-decisions/ADR-017-usearch-replaces-lancedb.md`
- **Файлы изменены:** `Cargo.toml` (workspace), `crates/retrieval/Cargo.toml`, `crates/retrieval/src/{lib,assembly,error}.rs`, `crates/cli/src/main.rs`, `crates/sub-lm/Cargo.toml`
- **Тесты:** 11 новых (embedding 2 + vector 5 + hybrid 3 + assembly 1 implicit), все проходят
- **Clippy:** 0 warnings
- **Результат:** Vector index полностью реализован, M4 ~50%
