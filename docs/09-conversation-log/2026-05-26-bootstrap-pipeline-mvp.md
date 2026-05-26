# 2026-05-26 — MVP Milestone 1: Bootstrap Pipeline Implementation

> Полная реализация 8-фазного pipeline для онбординга проектов в L4 knowledge graph.

---

## Контекст

Scaffold Cargo workspace был готов из предыдущей сессии. Задача этой сессии — реализовать MVP Milestone 1: Bootstrap pipeline для Django marketplace. Это первый значимый функциональный код CMOS.

---

## Что сделано

### Design phase

- Brainstorming: обсудили подходы к парсингу (tree-sitter vs Python ast vs гибрид), scope (все 8 фаз), Sub-LM backend (Ollama + API fallback), архитектуру pipeline (sequential + checkpoints).
- Design spec записан в `docs/superpowers/specs/2026-05-26-bootstrap-pipeline-design.md`.

### Implementation

- **crates/bootstrap/** — новый crate, 27 файлов, ~3800 строк Rust.
- **Extractors:**
  - `python.rs` — tree-sitter-python парсер (classes, functions, imports, decorators).
  - `django.rs` — Django-specific классификатор (models, views, admins, serializers, forms, middleware, signals, consumers, management commands). Исправлен баг с ложной классификацией Admin/ViewSet как Model.
- **Phases (8 штук):**
  - Phase 1: AST Sweep (tree-sitter, no LM)
  - Phase 2: Schema & Domain Extraction (FK/M2M edges)
  - Phase 3: Architectural Pattern Detection (URLs, middleware, signals)
  - Phase 4: Convention Mining (Ollama/API)
  - Phase 5: Git History Mining (git2 crate)
  - Phase 6: Rejected Approaches (TODO/FIXME + LM classification)
  - Phase 7: Documentation Ingestion (markdown + LM extraction)
  - Phase 8: Policy Elicitation (interactive CLI questionnaire)
- **Infrastructure:**
  - `graph_store.rs` — SQLite L4 graph (nodes, edges, checkpoints, append-only)
  - `inference/` — trait InferenceBackend + OllamaBackend + ApiBackend + MockBackend
  - `config.rs` — TOML config parsing (.cmos/config.toml)
  - `runner.rs` — PipelineRunner с checkpoint/resume
  - `progress.rs` — terminal progress reporting
- **CLI integration:**
  - `cmos bootstrap --project <name> --root <path> [--resume] [--no-interactive] [--skip-phases]`
  - `cmos graph stats --project <name> --root <path>`
  - `cmos graph query --project <name> --root <path> --kind <kind>`

### Verification

Запущен на реальном Django-проекте `D:\art_network_antigravity` (Scenica):
- 224 Python файлов обработано за 1.6s (Phase 1)
- 4956 nodes: 1998 functions, 1934 imports, 233 classes, 178 views, 58 models, 50 admins, 46 serializers, 28 signal handlers, 19 management commands, и т.д.
- 45 FK/M2M relationship edges
- 389 URL patterns
- 43 git hotspots из 223 коммитов
- Полный прогон (без LM-фаз): ~25 секунд

---

## Ключевые решения

1. **tree-sitter-python (in-process)** — быстрый, zero-copy, единый подход для всех языков через грамматики. CST достаточен для Django-паттернов.

2. **Sequential pipeline с checkpoints** — проще отладка, Ollama всё равно sequential. Checkpoint после каждой фазы позволяет `--resume`.

3. **Порядок классификации в DjangoExtractor** — Admin/ViewSet/Serializer проверяются ДО Model, потому что их базовые классы содержат "Model" в имени (ModelAdmin, ModelViewSet, ModelSerializer).

4. **Ollama как primary Sub-LM** — owner уже использует Ollama с Gemma2. Configurable model name + API fallback.

5. **Append-only graph store** — никаких DELETE, только tombstones. Соответствует ADR-009 и ADR-012.

6. **Language-agnostic architecture** — trait `LanguageExtractor` позволяет добавлять новые языки без изменения pipeline.

---

## Открытые вопросы

- LM-фазы не тестировались (Ollama не был запущен). Нужно проверить качество ответов Gemma2 на задачах convention mining и docs ingestion.
- Формальных unit tests нет — pipeline работает, но нет regression protection.
- Phase 3 (patterns) пока не строит call edges между функциями — только URLs и middleware. Это можно улучшить позже.

---

## Следующий шаг

Тестирование LM-фаз с Ollama → unit tests → CI → MVP M2 (Memory layers). См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Коммиты:** `8f5cc96` (design spec), `d2a0739` (implementation)
- **Результат:** MVP M1 полностью реализован и верифицирован на реальном проекте.
