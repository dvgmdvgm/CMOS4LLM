# NEXT — что делаем дальше

> **Конкретные следующие 3–5 шагов. Обновляется в конце каждой сессии.**
> 1 экран максимум. Если шаг сделан — двигаем его в STATUS, удаляем отсюда.

---

**По состоянию на:** 2026-05-26

## Очередь работы (в порядке выполнения)

### 1. ☐ Тестирование LM-фаз с Ollama

- [ ] Запустить Ollama, убедиться что Gemma2 доступна.
- [ ] Перезапустить bootstrap без `--skip-phases`: `cmos bootstrap --project marketplace --root D:\art_network_antigravity --no-interactive --resume`.
- [ ] Проверить что фазы 4 (Convention Mining), 6 (Rejected Approaches), 7 (Docs Ingestion) отрабатывают.
- [ ] Проверить качество LM-ответов (conventions, tech debt markers, doc facts).

### 2. ☐ Unit tests для bootstrap pipeline

- [ ] Создать `crates/bootstrap/tests/fixtures/` с синтетическими Django-файлами.
- [ ] Unit tests для PythonExtractor (parse_file → correct RawNodes).
- [ ] Unit tests для DjangoExtractor (classify_node → correct kinds).
- [ ] Unit tests для GraphStore (insert/query/checkpoint).
- [ ] Integration test: full pipeline на mini-Django fixture.

### 3. ☐ CI pipeline (GitHub Actions)

- [ ] `cargo clippy` + `cargo test` + `cargo build --release` на Windows runner.
- [ ] Frontend: `pnpm install` + `pnpm build`.
- [ ] Создать GitHub repo и push.

### 4. ☐ MVP Milestone 2: Memory layers L1–L4

- [ ] L1: in-memory prompt assembly buffer.
- [ ] L2/L3: SQLite WAL event store (schema из ADR-016).
- [ ] L4: интеграция с существующим GraphStore из bootstrap.
- [ ] Promotion logic L2→L3, L3→L4.

### 5. ☐ MVP Milestone 3: Two-LLM economy

- [ ] Интеграция Ollama для runtime inference (не только bootstrap).
- [ ] Background task queue (tokio channels).
- [ ] Context assembly: retrieval из L4 graph → prompt для Cloud LLM.

---

## Если ты только что открыл этот файл

- MVP M1 (Bootstrap pipeline) полностью реализован и работает.
- Pipeline извлекает ~5000 nodes из реального Django-проекта за 25 секунд.
- LM-фазы (Ollama) ещё не тестировались — нужно запустить Ollama.
- Следующий шаг — тесты или сразу M2 (memory layers).

---

## Ритуал в конце сессии

Перед закрытием окна обязательно (см. `CLAUDE.md` — раздел SLEEP RITUAL):
1. Перенести выполненные шаги отсюда → в `STATUS.md`.
2. Добавить новые шаги, если появились.
3. Если приняты архитектурные решения → новый ADR в `docs/02-decisions/`.
4. Создать новый файл в `docs/09-conversation-log/YYYY-MM-DD-<topic>.md`.
5. Закрыть открытые вопросы или перенести в ROADBLOCKS.

Или просто напиши `/sleep` — Claude прогонит ритуал автоматически.
