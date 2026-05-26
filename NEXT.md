# NEXT — что делаем дальше

> **Конкретные следующие 3–5 шагов. Обновляется в конце каждой сессии.**
> 1 экран максимум. Если шаг сделан — двигаем его в STATUS, удаляем отсюда.

---

**По состоянию на:** 2026-05-26

## Очередь работы (в порядке выполнения)

### 1. ☐ Git commit scaffold

- [ ] `git add` всех новых файлов (crates, apps, run.bat, .cargo).
- [ ] Commit: "feat: scaffold Cargo workspace + Tauri desktop app".
- [ ] Создать GitHub repo и push (если owner готов).

### 2. ☐ CI pipeline (GitHub Actions)

- [ ] `cargo clippy` + `cargo test` + `cargo build --release` на Windows runner.
- [ ] Frontend: `pnpm install` + `pnpm build`.
- [ ] Создать GitHub repo и push.

### 3. ☐ MVP Milestone 1: Bootstrap pipeline для Django marketplace

Первый исполнимый код: статический анализ Django-проекта → построение L4 symbol graph + domain ontology.
- [ ] Python AST parser (вызывается из Rust через `tree-sitter-python` или subprocess).
- [ ] Django-specific extractors (models, views, urls, signals).
- [ ] SQLite schema для L4 graph (nodes + edges tables).
- [ ] CLI: `cmos bootstrap --project marketplace --root <path>`.

### 4. ☐ MVP Milestone 2: Memory layers L1–L4

- [ ] L1: in-memory prompt assembly buffer.
- [ ] L2/L3: SQLite WAL event store (schema из ADR-016).
- [ ] L4: SQLite graph (schema из ADR-012) + LanceDB vectors.
- [ ] Promotion logic L2→L3, L3→L4.

### 5. ☐ MVP Milestone 3: Two-LLM economy

- [ ] llama-cpp-2 integration: load GGUF model, run classification task.
- [ ] Background task queue (tokio channels).
- [ ] Fallback to Haiku API when no GPU.

---

## Если ты только что открыл этот файл

- Scaffold готов и проверен. Все crates компилируются, Tauri app собирается и запускается.
- `run.bat` автоматически ставит все зависимости на чистой машине.
- Следующий шаг — **git commit** (пункт 1), затем CI или сразу bootstrap pipeline.
- Открытые вопросы в [ROADBLOCKS.md](./ROADBLOCKS.md) не блокируют текущую работу.

---

## Ритуал в конце сессии

Перед закрытием окна обязательно (см. `CLAUDE.md` — раздел SLEEP RITUAL):
1. Перенести выполненные шаги отсюда → в `STATUS.md`.
2. Добавить новые шаги, если появились.
3. Если приняты архитектурные решения → новый ADR в `docs/02-decisions/`.
4. Создать новый файл в `docs/09-conversation-log/YYYY-MM-DD-<topic>.md`.
5. Закрыть открытые вопросы или перенести в ROADBLOCKS.

Или просто напиши `/sleep` — Claude прогонит ритуал автоматически.
