# NEXT — что делаем дальше

> **Конкретные следующие 3–5 шагов. Обновляется в конце каждой сессии.**
> 1 экран максимум. Если шаг сделан — двигаем его в STATUS, удаляем отсюда.

---

**По состоянию на:** 2026-05-26

## Очередь работы (в порядке выполнения)

### 1. ☐ Push all uncommitted changes на GitHub

- [ ] Commit всех новых и изменённых файлов (M5 GUI + Token Analytics + portable build).
- [ ] Push на main.
- [ ] Убедиться что CI green.

### 2. ☐ Token Analytics — baseline measurement

- [ ] Запустить CMOS MCP server + Claude на реальном проекте (Django marketplace).
- [ ] Собрать данные: tokens assembled vs baseline estimate за 10+ запросов.
- [ ] Проверить, что savings ratio отображается в GUI корректно.
- [ ] Задокументировать результат в README (секция "Benchmarks").

### 3. ☐ Документация setup для Claude Desktop/Code

- [ ] Финализировать README секцию "Usage with Claude" (частично сделана).
- [ ] Пример конфига для Claude Desktop (`claude_desktop_config.json`).
- [ ] Пример конфига для Claude Code (`.claude/settings.json`).

### 4. ☐ GUI — визуальная проверка на реальных данных

- [ ] Запустить `run.bat dev`, убедиться что UI рендерится с реальными данными.
- [ ] Проверить все панели: Stats, Facts, Events, Tokens.
- [ ] Исправить визуальные баги если есть.

---

## Если ты только что открыл этот файл

- MVP M1–M5 полностью реализованы.
- Token Analytics инструментирован в MCP gateway (assemble_context записывает в SQLite).
- GUI skeleton: Tauri + React + Zustand + Tailwind v4. Sidebar (projects) + tabs (Stats, Facts, Events, Tokens).
- 95 тестов проходят, CI green, clippy 0 warnings.
- MCP transport: newline-delimited JSON.
- GUI работает in-process (без daemon) — Tauri commands вызывают cmos-memory/cmos-retrieval напрямую.
- **Portable:** данные в `data/` в корне проекта (env var `CMOS_DATA_DIR`). Никаких AppData зависимостей.
- **run.bat** — one-click build & launch, автоустановка всех зависимостей включая WebView2.

---

## Ритуал в конце сессии

Перед закрытием окна обязательно (см. `CLAUDE.md` — раздел SLEEP RITUAL):
1. Перенести выполненные шаги отсюда → в `STATUS.md`.
2. Добавить новые шаги, если появились.
3. Если приняты архитектурные решения → новый ADR в `docs/02-decisions/`.
4. Создать новый файл в `docs/09-conversation-log/YYYY-MM-DD-<topic>.md`.
5. Закрыть открытые вопросы или перенести в ROADBLOCKS.

Или просто напиши `/sleep` — Claude прогонит ритуал автоматически.
