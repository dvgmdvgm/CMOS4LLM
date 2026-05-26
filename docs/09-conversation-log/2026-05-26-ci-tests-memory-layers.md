# 2026-05-26 — CI, Tests, and MVP Milestone 2 (Memory Layers)

> Тестирование LM-фаз, unit tests, CI pipeline, push на GitHub, и полная реализация memory layers L1–L4.

---

## Контекст

MVP Milestone 1 (Bootstrap pipeline) был полностью реализован в предыдущей сессии. Pipeline извлекает ~5000 nodes из реального Django-проекта. LM-фазы были написаны, но не тестировались с Ollama. Формальных тестов не было. GitHub repo не существовал.

---

## Что сделано

### Тестирование LM-фаз
- Ollama запущена с Gemma4 (вместо Gemma2 из дефолтов).
- `.cmos/config.toml` в тестовом проекте уже указывал `gemma4:latest`.
- Все LM-фазы верифицированы: 5 conventions, 21 tech_debt_markers, 22 doc_facts + 240 domain_terms + 147 constraints + 143 architectural_decisions.
- Прямой тест Ollama API подтвердил корректный JSON output.

### Unit tests (`crates/bootstrap/tests/`)
- 10 fixture файлов: views.py, models.py, admin.py, serializers.py, signals.py, middleware.py, consumers.py, management_command.py, urls.py, settings.py.
- `test_python_extractor.rs` — 9 тестов (functions, classes, imports, bases, decorators, line numbers).
- `test_django_extractor.rs` — 15 тестов (views CBV/FBV, admins, serializers, models, signals, middleware, consumers, URLs, model fields, settings middleware).
- `test_graph_store.rs` — 9 тестов (insert/query, batch, edges, checkpoints, project isolation).

### CI pipeline (`.github/workflows/ci.yml`)
- Rust job: clippy + test + build (Windows runner).
- Frontend job: pnpm install + build (apps/desktop/).
- 25 clippy warnings исправлены (collapsible_if, io_other_error, unnecessary_map_or, и др.).

### GitHub
- Repo создан owner'ом: `dvgmdvgm/CMOS4LLM`.
- Remote добавлен, все коммиты запушены на `main`.

### MVP Milestone 2: Memory layers (`crates/memory/`)
- **L1 Working Memory** (`l1.rs`): VecDeque-based buffer с RwLock, priority-based eviction (System > Policy > Context > Scratch), token budgeting, `assemble()` и `assemble_within_budget()`.
- **L2/L3 Event Store** (`l2l3.rs`): SQLite WAL, immutable events, 8 event types, temporal/session/entity/layer/type queries, access counting, `promote_to_l3()`, `candidates_for_promotion()`.
- **L4 Project Memory** (`l4.rs`): persistent facts с tombstones, FactSource tracking (Bootstrap/Promotion/UserDeclared/Inferred), kind/label queries.
- **Promotion Engine** (`promotion.rs`): configurable thresholds, `run_l2_to_l3()` и `run_l3_to_l4()`, event-to-fact conversion.
- **30 тестов** в 4 файлах: test_l1.rs (9), test_l2l3.rs (10), test_l4.rs (6), test_promotion.rs (5).

---

## Ключевые решения

1. **Gemma4 вместо Gemma2** — owner уже имеет gemma4:latest в Ollama. Config в `.cmos/config.toml` переопределяет дефолт из кода. Код проверяет модель через `starts_with` по имени до двоеточия.

2. **L1 как VecDeque + RwLock** — не lock-free в строгом смысле (RwLock), но достаточно для single-process desktop app. Eviction по composite score: priority_weight + recency + access_count.

3. **L2/L3 в одной таблице с `layer` column** — проще queries, один файл events.db. Promotion = UPDATE layer. Разделение на отдельные файлы (как обсуждалось в ADR-016) отложено до момента, когда access patterns покажут необходимость.

4. **L4 отдельно от GraphStore** — L4 ProjectMemory хранит "promoted facts" (decisions, lessons), а GraphStore из bootstrap хранит code structure (nodes, edges). Это два разных аспекта L4: code ontology vs. project knowledge. В будущем нужен unified query interface.

5. **Promotion thresholds как config** — дефолты (access≥3/importance≥0.6 для L2→L3, access≥5/importance≥0.8 для L3→L4) — стартовая точка. Будут тюниться после реального использования.

---

## Открытые вопросы

- Как интегрировать L4 ProjectMemory с L4 GraphStore из bootstrap? Нужен unified retrieval interface.
- Vector embeddings для semantic search в L3/L4 — какую модель использовать через Ollama? (nomic-embed-text? mxbai-embed-large?)
- Context assembly budget: сколько токенов из каждого layer включать в prompt? (ADR-002 говорит 16K–64K total, Q2 в ROADBLOCKS).

---

## Следующий шаг

MVP Milestone 3: Two-LLM economy — runtime inference + context assembly. См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Коммиты:** `ac99a4a` (tests), `d5f9781` (CI + clippy), `a26e017` (M2 memory layers)
- **Результат:** M2 полностью реализован, CI настроен, GitHub repo live.
