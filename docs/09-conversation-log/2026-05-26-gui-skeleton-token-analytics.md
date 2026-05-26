# 2026-05-26 — GUI Skeleton + Token Analytics

> MVP Milestone 5 закрыт. Token Analytics инструментирован в MCP gateway.

---

## Контекст

MVP Milestones 1–4 были полностью реализованы: bootstrap, memory layers L1–L4, two-LLM economy, vector index + MCP server. 95 тестов проходили, CI green. Следующий шаг — GUI skeleton (Tauri + React) и Token Analytics.

---

## Что сделано

### Tauri Commands (backend)
- `apps/desktop/src-tauri/src/main.rs` — полностью переписан. 6 команд:
  - `get_version` — версия CMOS.
  - `get_memory_stats` — counts по L1/L2/L3/L4 + vector index.
  - `list_projects` — distinct project_id из events.db + facts.db.
  - `get_facts` — L4 факты с фильтрацией по kind.
  - `get_events` — L2/L3 события с фильтрацией по layer.
  - `get_token_stats` — читает token_analytics.db (shared с MCP gateway).
- `apps/desktop/src-tauri/Cargo.toml` — добавлены cmos-memory, cmos-retrieval, rusqlite, dirs, tokio.

### Frontend (React + Zustand + Tailwind v4)
- `apps/desktop/src/store.ts` — Zustand store: projects, stats, facts, events, tokenStats, activeTab.
- `apps/desktop/src/App.tsx` — layout: sidebar + tab bar + content panels.
- `apps/desktop/src/components/Sidebar.tsx` — project list с counts.
- `apps/desktop/src/components/StatsPanel.tsx` — memory stats cards + layer distribution bar.
- `apps/desktop/src/components/FactsPanel.tsx` — таблица L4 фактов с kind badges.
- `apps/desktop/src/components/EventsPanel.tsx` — список L2/L3 событий с layer/type badges.
- `apps/desktop/src/components/TokensPanel.tsx` — token analytics dashboard (savings ratio, metrics).
- `apps/desktop/src/styles.css` — Tailwind v4 с custom dark theme (surface-0..3, accent, etc.).
- `apps/desktop/vite.config.ts` — добавлен @tailwindcss/vite plugin.
- `apps/desktop/package.json` — добавлены zustand, tailwindcss, @tailwindcss/vite.

### Token Analytics (instrumentation)
- `crates/gateway/src/analytics.rs` — новый модуль `TokenTracker`:
  - SQLite persistence в `token_analytics.db`.
  - Atomic counters для in-memory fast path.
  - `record()` — записывает tokens_assembled vs tokens_baseline_estimate.
  - `stats()` — возвращает агрегированную статистику.
- `crates/gateway/src/handler.rs` — `assemble_context` инструментирован: после сборки контекста записывает в TokenTracker.
- `crates/gateway/src/server.rs` — TokenTracker инициализируется при старте MCP server.
- `crates/gateway/src/lib.rs` — экспортирует analytics модуль.
- `crates/gateway/Cargo.toml` — добавлен rusqlite.

---

## Ключевые решения

1. **GUI работает in-process, без daemon.** ADR-007 описывает daemon на localhost:7077, но для MVP это overkill. Tauri commands вызывают cmos-memory/cmos-retrieval напрямую через Rust. Daemon — V1+.

2. **Token Analytics: shared SQLite.** MCP gateway пишет в `token_analytics.db`, desktop app читает тот же файл. Простое решение для MVP — оба процесса используют WAL mode, конфликтов нет (один writer, один reader).

3. **Baseline estimate — эвристика.** `items_considered * 200 + task_description_len / 4`. Это грубая оценка "сколько токенов было бы без CMOS". Точный baseline требует A/B тестирования на реальных задачах (следующий шаг).

4. **Tailwind v4 (не v3).** Используем `@import "tailwindcss"` + `@theme {}` вместо tailwind.config.js. Проще, меньше файлов, нативная интеграция с Vite через @tailwindcss/vite.

5. **Zustand без middleware.** Для MVP достаточно простого store без persist/devtools. Добавим по необходимости.

---

## Открытые вопросы

- Нет новых. Все существующие Q1–Q9 в ROADBLOCKS.md остаются актуальными.

---

## Следующий шаг

- Token Analytics baseline measurement на реальном проекте.
- Документация setup для Claude Desktop/Code.
- Визуальная проверка GUI (`npm run tauri dev`).
- Push на GitHub.

См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы созданы:** `analytics.rs`, `store.ts`, `Sidebar.tsx`, `StatsPanel.tsx`, `FactsPanel.tsx`, `EventsPanel.tsx`, `TokensPanel.tsx`
- **Файлы изменены:** `main.rs` (desktop), `handler.rs`, `server.rs`, `lib.rs` (gateway), `Cargo.toml` (gateway, desktop), `vite.config.ts`, `styles.css`, `App.tsx`, `package.json`
- **Тесты:** 95 проходят, clippy 0 warnings, TypeScript clean
- **Результат:** MVP ~92%, Milestone 5 + Token Analytics закрыты
