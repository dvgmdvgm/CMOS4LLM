# 2026-05-26 — Bootstrap-Event Integration & Retrieval Quality

> Интеграция bootstrap pipeline с event store + retrieval quality tests + CI push.

---

## Контекст

M4 (Vector index + MCP) был завершён в предыдущей сессии. Оставались 4 пункта в NEXT.md: bootstrap→events, retrieval quality tests, CI improvements, MCP hybrid assembly. Эта сессия закрыла первые три.

---

## Что сделано

### Bootstrap → Event Store интеграция

- `crates/bootstrap/Cargo.toml` — добавлена зависимость `cmos-memory`.
- `crates/bootstrap/src/context.rs` — `PipelineContext` получил поле `event_store: Option<EventStore>`.
- `crates/bootstrap/src/runner.rs` — runner открывает `events.db` при старте, после каждой успешной фазы вызывает `emit_phase_event()` → L2 `Extraction` event с payload (phase name, nodes_created, edges_created, warnings count).
- `crates/bootstrap/tests/test_event_integration.rs` — 3 теста: events emitted, payload structure, skipped phases silent.

### Retrieval quality tests

- `crates/retrieval/tests/test_retrieval_quality.rs` — 7 тестов:
  - `keyword_retrieval_l4_precision` — precision@10 ≥ 0.2 на synthetic corpus.
  - `keyword_retrieval_l3_recall` — recall@10 ≥ 0.3.
  - `hybrid_retrieval_l4_outperforms_keyword_only` — vector search находит семантически релевантные факты.
  - `hybrid_retrieval_l3_vector_finds_semantic_matches` — vector search для L3 эпизодов.
  - `budget_enforcement_under_200ms_equivalent` — 100 итераций < 200ms per call.
  - `assembled_context_respects_budget` — бюджет не нарушается (500–8000 tokens).
  - `higher_importance_items_ranked_first` — high-confidence items ранжируются выше.
- Synthetic corpus: 20 L4 facts + 15 L3 episodes, 8-dimensional embeddings, 4 relevance judgments.

### CI improvements

- `README.md` — добавлен CI badge.
- Cargo caching уже было (`Swatinem/rust-cache@v2`).
- Push на GitHub: 3 коммита (M3, M4, текущая сессия).

---

## Ключевые решения

1. **Event emission — fire-and-forget, не блокирует bootstrap.** Если event store не открылся (например, нет прав на запись), bootstrap продолжает работать. `tracing::warn` при ошибке записи. Причина: bootstrap — критический путь, event store — observability, не должен ломать основной flow.

2. **Skipped/failed фазы не эмитят events.** Только успешно завершённые фазы записывают L2 Extraction. Причина: event store хранит «что произошло», а не «что не произошло». Failures видны в graph.db checkpoints.

3. **Synthetic embeddings для retrieval quality tests (dim=8).** Не используем реальный Ollama в тестах — детерминизм важнее реалистичности. Embeddings вручную сконструированы так, чтобы отражать семантические кластеры (database, security, api, perf, code, ops). Причина: тесты должны быть reproducible и fast, без внешних зависимостей.

4. **Precision/recall thresholds консервативные (0.2/0.3).** Keyword-only retrieval не использует query text для scoring (нет TF-IDF), поэтому precision зависит только от importance/recency/access. Thresholds отражают baseline — при добавлении query-aware scoring они должны расти.

---

## Открытые вопросы

- VectorIndex Send-safety для MCP hybrid assembly — единственный оставшийся технический долг в M4.
- Retrieval quality с реальными embeddings (Ollama) не тестировалась — только synthetic.

---

## Следующий шаг

MCP hybrid assembly (VectorIndex Send-safety). См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы созданы:** `crates/bootstrap/tests/test_event_integration.rs`, `crates/retrieval/tests/test_retrieval_quality.rs`
- **Файлы изменены:** `crates/bootstrap/{Cargo.toml, src/context.rs, src/runner.rs}`, `README.md`
- **Тесты:** 93 проходят (было 83), +3 bootstrap-event, +7 retrieval quality
- **Clippy:** 0 warnings
- **Git:** 3 коммита запушены на GitHub (M3 + M4 + integration)
- **Результат:** NEXT.md пункты 1–3 закрыты, MVP ~75%
