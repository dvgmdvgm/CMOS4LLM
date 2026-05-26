# 2026-05-26 — MVP Milestone 3: Two-LLM Economy

> Sub-LM runtime, background task queue, context assembly engine, CLI integration.

---

## Контекст

MVP Milestone 2 (Memory layers L1–L4) был полностью реализован в предыдущей сессии. Memory layers работали изолированно — не было runtime inference, background processing, и context assembly. Следующий шаг по roadmap — M3: Two-LLM economy.

---

## Что сделано

### `crates/sub-lm/` — Sub-LM runtime service (новый)
- `src/service.rs` — `InferenceService` trait: `complete`, `classify`, `summarize`, `extract_json`, `health_check`.
- `src/ollama.rs` — `OllamaRuntime`: production Ollama backend (retry с exponential backoff, health check через `/api/tags`, configurable timeout/model/endpoint).
- `src/queue.rs` — `TaskQueue`: tokio mpsc worker pool, `TaskKind` enum (Summarize/ExtractJson/Classify/Complete), priority support, async `submit`/`submit_and_wait`, counters.
- `src/error.rs` — `SubLmError` enum.
- `Cargo.toml` — добавлены `reqwest`, `async-trait`, `chrono`, `serde_json`.
- 4 unit теста (mock service, parallel execution, counters).

### `crates/retrieval/` — Context assembly engine (переписан из placeholder)
- `src/assembly.rs` — `ContextAssembler`: budget-aware сборка контекста из L1+L3+L4. Budget split: 40% L4, 40% L3, 20% L1. `ContextQuery` с builder pattern. `AssembledContext` с `render()` и `render_with_header()`.
- `src/scoring.rs` — `RelevanceScorer`: weighted scoring (recency 0.4 + importance 0.4 + access 0.2). Token estimation.
- `src/error.rs` — `RetrievalError`.
- `Cargo.toml` — добавлены `cmos-memory`, `serde_json`.
- 5 unit тестов (empty, L4-only, L1-only, budget enforcement, render).

### `crates/cli/src/main.rs` — новые CLI команды
- `cmos context --project X --root Y --task "desc" [--budget N] [--session S]` — собирает и выводит контекст.
- `cmos memory stats --project X --root Y` — статистика L2/L3/L4.
- `cmos memory query --project X --root Y [--layer L3] [--type decision] [--limit 20]` — поиск по event store.
- `cmos memory promote --project X --root Y` — ручной запуск promotion engine.
- `Cargo.toml` — добавлены `cmos-memory`, `cmos-retrieval`, `cmos-sub-lm`, `serde_json`.

---

## Ключевые решения

1. **Sub-LM как отдельный crate, не часть bootstrap** — bootstrap зависит от sub-lm (пока не переключён, но API совместим). Это позволяет gateway, CLI, и будущему MCP server использовать один и тот же inference service.

2. **TaskQueue на tokio mpsc, не priority queue** — для MVP достаточно FIFO с priority enum. Настоящий priority scheduling (heap-based) добавим когда появится реальная нагрузка с mixed priorities. Сейчас worker count = 2 (один inference за раз на GPU, второй в очереди).

3. **Context assembly budget split 40/40/20** — L4 (project knowledge) и L3 (episodes) получают равные доли, L1 (working memory) меньше, т.к. она уже в текущем контексте сессии. Это стартовая точка, будет тюниться.

4. **Retrieval без vector index** — в M3 retrieval работает на keyword/structural basis (query by kind, layer, session). Vector similarity (LanceDB + embeddings) — отдельный milestone. Это сознательное решение: сначала working pipeline, потом quality improvements.

5. **CLI sync, не async** — `cmos context` и `cmos memory` работают синхронно (блокирующие SQLite queries). Async нужен только для Sub-LM calls. Это упрощает CLI code и достаточно для desktop use case.

---

## Открытые вопросы

- Как интегрировать Sub-LM crate обратно в bootstrap (заменить дублирующийся OllamaBackend)? Нужен рефакторинг bootstrap → зависимость от sub-lm.
- Embedding model для vector retrieval: nomic-embed-text vs mxbai-embed-large через Ollama.
- MCP protocol: какую версию спецификации таргетить (2024-11-05?).

---

## Следующий шаг

Vector index (LanceDB) + MCP Server. См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы созданы:** `crates/sub-lm/src/{service,ollama,queue,error}.rs`, `crates/retrieval/src/{assembly,scoring,error}.rs`
- **Файлы изменены:** `crates/sub-lm/Cargo.toml`, `crates/retrieval/Cargo.toml`, `crates/cli/Cargo.toml`, `crates/cli/src/main.rs`, `crates/sub-lm/src/lib.rs`, `crates/retrieval/src/lib.rs`
- **Тесты:** 9 новых (4 sub-lm + 5 retrieval), все проходят
- **Clippy:** 0 warnings
- **Результат:** M3 полностью реализован
