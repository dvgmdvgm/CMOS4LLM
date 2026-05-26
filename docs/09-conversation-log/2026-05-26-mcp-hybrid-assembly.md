# 2026-05-26 — MCP Hybrid Assembly (VectorIndex Send-safety)

> Решение проблемы !Sync для VectorIndex и переключение MCP на hybrid retrieval.

---

## Контекст

M4 (Vector index + MCP) был завершён, но MCP handler использовал keyword-only assembly из-за того, что VectorIndex содержит `rusqlite::Connection`, который `!Sync`. Ссылка `&VectorIndex` через await point требует `Send`, а `&T: Send` требует `T: Sync`. Это блокировало использование `assemble_hybrid` в async MCP handler.

---

## Что сделано

### VectorIndex → Mutex-обёртка
- `crates/retrieval/src/vector.rs` — `meta_db` обёрнут в `std::sync::Mutex<SqliteConn>`, `next_key` в `Mutex<u64>`.
- `upsert()` теперь принимает `&self` вместо `&mut self` (interior mutability через Mutex).
- `search()` берёт lock на `meta_db` только на время SQL-запросов.
- Использует `unchecked_transaction()` вместо `transaction()` (последний требует `&mut self` на Connection).

### Assembly — разделение async/sync
- `crates/retrieval/src/assembly.rs` — добавлен `assemble_hybrid_with_embedding()`:
  - Синхронный метод, принимает готовый `&[f32]` embedding.
  - Не держит `!Sync` ссылки через await points.
- `assemble_hybrid()` теперь: embed query (async) → вызов `assemble_hybrid_with_embedding()` (sync).

### HybridRetriever — синхронные варианты
- `crates/retrieval/src/hybrid.rs` — добавлены:
  - `retrieve_l4_with_embedding()` — синхронный, принимает готовый embedding.
  - `retrieve_l3_with_embedding()` — аналогично.
- Оригинальные async методы теперь делегируют в `*_with_embedding()`.

### MCP Handler
- `crates/gateway/src/handler.rs` — `handle_assemble_context`:
  - `if let Some(ref vi) = vi` → embed query (async, не держит !Sync refs) → `assemble_hybrid_with_embedding` (sync).
  - Fallback на keyword-only `assemble()` если vector index недоступен.

### Cleanup
- Убраны все `mut` bindings для VectorIndex (vector.rs tests, test_retrieval_quality.rs, cli/main.rs).
- Удалён временный `send_check.rs` test file.

---

## Ключевые решения

1. **Mutex вместо Arc<Mutex> + spawn_blocking.** Проще, меньше overhead. VectorIndex создаётся per-request в handler (не shared state), поэтому contention невозможен. Mutex нужен только для Sync trait bound, реально lock никогда не contended.

2. **Разделение async embedding от sync retrieval.** Альтернатива — обернуть весь assemble_hybrid в spawn_blocking. Отвергнуто: spawn_blocking не может вызывать async (embed_single), пришлось бы создавать nested runtime. Разделение чище: один await для embedding, потом чистый sync код.

3. **unchecked_transaction() вместо transaction().** `rusqlite::Connection::transaction()` требует `&mut self`. Через `Mutex<Connection>` мы получаем `MutexGuard` (даёт `&Connection`, не `&mut Connection`). `unchecked_transaction()` работает с `&self` — безопасно, т.к. Mutex гарантирует exclusive access.

4. **upsert(&self) вместо upsert(&mut self).** Interior mutability через Mutex. Breaking change для callers (убрать `mut`), но тривиальный — clippy сам подсказывает.

---

## Открытые вопросы

- Нет новых. Все Q1-Q9 в ROADBLOCKS остаются без изменений.

---

## Следующий шаг

End-to-end integration test с реальным MCP client. См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы изменены:** `crates/retrieval/src/vector.rs`, `crates/retrieval/src/assembly.rs`, `crates/retrieval/src/hybrid.rs`, `crates/gateway/src/handler.rs`, `crates/cli/src/main.rs`, `crates/retrieval/tests/test_retrieval_quality.rs`
- **Тесты:** 93 проходят (без изменений в количестве), clippy 0 warnings
- **Результат:** NEXT.md пункт 1 (MCP hybrid assembly) закрыт, MVP ~80%
