# NEXT — что делаем дальше

> **Конкретные следующие 3–5 шагов. Обновляется в конце каждой сессии.**
> 1 экран максимум. Если шаг сделан — двигаем его в STATUS, удаляем отсюда.

---

**По состоянию на:** 2026-05-26

## Очередь работы (в порядке выполнения)

### 1. ☐ MVP Milestone 3: Two-LLM economy

- [ ] Интеграция Ollama для runtime inference (не только bootstrap).
- [ ] Background task queue (tokio channels) для Sub-LM задач.
- [ ] Context assembly: retrieval из L4 graph + L3 episodes → prompt для Cloud LLM.
- [ ] CLI команда `cmos context` — показать собранный контекст для текущей задачи.

### 2. ☐ Интеграция memory layers с CLI

- [ ] `cmos memory stats` — показать L1/L2/L3/L4 статистику.
- [ ] `cmos memory query --layer L3 --type decision` — поиск по event store.
- [ ] `cmos memory promote` — ручной запуск promotion engine.
- [ ] Автоматический append в L2 при каждом вызове `cmos bootstrap`.

### 3. ☐ Vector index (LanceDB) для semantic retrieval

- [ ] Интеграция LanceDB в crates/retrieval.
- [ ] Embedding generation через Ollama (nomic-embed-text или similar).
- [ ] Hybrid retrieval: vector similarity + graph traversal.

### 4. ☐ MCP Server (ADR-010)

- [ ] Реализовать MCP protocol handler в crates/gateway.
- [ ] Expose memory layers через MCP tools.
- [ ] Тестирование с Claude Desktop / Claude Code.

### 5. ☐ CI improvements

- [ ] Добавить caching для Cargo build в GitHub Actions.
- [ ] Добавить badge в README.

---

## Если ты только что открыл этот файл

- MVP M1 (Bootstrap) и M2 (Memory layers) полностью реализованы.
- GitHub repo: `dvgmdvgm/CMOS4LLM`, CI настроен.
- Следующий шаг — M3 (Two-LLM economy): runtime inference + context assembly.
- Memory layers работают изолированно, но ещё не интегрированы с CLI.

---

## Ритуал в конце сессии

Перед закрытием окна обязательно (см. `CLAUDE.md` — раздел SLEEP RITUAL):
1. Перенести выполненные шаги отсюда → в `STATUS.md`.
2. Добавить новые шаги, если появились.
3. Если приняты архитектурные решения → новый ADR в `docs/02-decisions/`.
4. Создать новый файл в `docs/09-conversation-log/YYYY-MM-DD-<topic>.md`.
5. Закрыть открытые вопросы или перенести в ROADBLOCKS.

Или просто напиши `/sleep` — Claude прогонит ритуал автоматически.
