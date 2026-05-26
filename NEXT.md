# NEXT — что делаем дальше

> **Конкретные следующие 3–5 шагов. Обновляется в конце каждой сессии.**
> 1 экран максимум. Если шаг сделан — двигаем его в STATUS, удаляем отсюда.

---

**По состоянию на:** 2026-05-26

## Очередь работы (в порядке выполнения)

### 1. ☐ Технические ADR (выбор стека)

Architecture.md закрыл «что», теперь ADR-011..016 закроют «чем».
- [ ] ADR-011: Backend язык (Rust vs Go) для core daemon.
- [ ] ADR-012: Graph DB для L4 (Kuzu vs Memgraph vs CozoDB embedded).
- [ ] ADR-013: Vector index (Qdrant vs Lance vs hnswlib).
- [ ] ADR-014: Sub-LM runtime (llama.cpp vs vLLM vs MLX, выбор моделей).
- [ ] ADR-015: GUI shell (Tauri 2.x), frontend stack.
- [ ] ADR-016: Storage backend для L2/L3 event log (RocksDB vs SQLite vs custom).

### 2. ☐ Стартовый репозиторий + CI

- [ ] `git init` в `D:\AI Projects\CMOS\`.
- [ ] Базовая структура исходников (`crates/` для Rust core, `apps/desktop` для Tauri, `apps/web` для standalone web GUI).
- [ ] CI pipeline (GitHub Actions): lint + test + build на каждом push.

### 3. ☐ MVP Milestone 1: Bootstrap-pipeline для Django marketplace

После того как scope зафиксирован и стек выбран — **первый исполнимый код**: статический анализ Django-проекта → построение L4 symbol graph + domain ontology.

### 4. ☐ MVP Milestone 2: Memory layers L1–L4

Реализация хранилищ по выбранному стеку (ADR-012..016). Promotion logic, tombstones, persistence across restarts.

### 5. ☐ MVP Milestone 3: Two-LLM economy

Sub-LM Runtime: пул одной локальной модели, background queue, fallback to Haiku.

---

## Если ты только что открыл этот файл

- Если шаги выглядят знакомо и понятно → бери шаг 1 и работай.
- Если потерялся в архитектуре → возвращайся к [docs/01-architecture.md](./docs/01-architecture.md) → последний conversation-log в `docs/09-conversation-log/`.
- Если есть открытые вопросы — они в [ROADBLOCKS.md](./ROADBLOCKS.md), их надо решить **до** начала имплементации.

---

## Ритуал в конце сессии

Перед закрытием окна обязательно (см. `CLAUDE.md` — раздел SLEEP RITUAL):
1. Перенести выполненные шаги отсюда → в `STATUS.md`.
2. Добавить новые шаги, если появились.
3. Если приняты архитектурные решения → новый ADR в `docs/02-decisions/`.
4. Создать новый файл в `docs/09-conversation-log/YYYY-MM-DD-<topic>.md`.
5. Закрыть открытые вопросы или перенести в ROADBLOCKS.

Или просто напиши `/sleep` — Claude прогонит ритуал автоматически.
