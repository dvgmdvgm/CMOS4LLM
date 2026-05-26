# STATUS — где мы сейчас

> **Этот файл обновляется в конце каждой сессии работы над CMOS. 1 экран максимум.**
> Если STATUS не обновлён — следующая сессия начнётся со слепого понимания.

---

**Дата последнего обновления:** 2026-05-26
**Текущая фаза:** MVP implementation — portable build & launch verified (~93% MVP)
**Кто работал:** owner + Opus 4.6

---

## Что сделано в последнюю сессию

1. **Портативность приложения:**
   - `apps/desktop/src-tauri/src/main.rs` — data_root переключён с `dirs::data_local_dir()` (AppData) на portable: `CMOS_DATA_DIR` env var → fallback рядом с exe. Зависимость `dirs` удалена.
   - `apps/desktop/src-tauri/Cargo.toml` — убрана зависимость `dirs = "6"`.
   - `.gitignore` — добавлена запись `data/`.
2. **run.bat — полная автоматизация:**
   - Добавлен `%USERPROFILE%\.cargo\bin` в PATH в начале (решает cargo not in PATH).
   - Добавлена автоустановка WebView2 Runtime через Microsoft bootstrapper (не winget).
   - Frontend deps: всегда `pnpm install` (frozen-lockfile → full install fallback).
   - Release-режим теперь тоже запускает exe после сборки.
   - `CMOS_DATA_DIR=%~dp0data` задаётся перед сборкой — данные всегда в корне проекта.
3. **WebView2 Runtime установлен** — приложение запускается, окно "CMOS Cognitive Console" отображается.
4. **Все 95 тестов проходят, clippy clean, TypeScript clean.**

## Что НЕ сделано (ждёт следующей сессии)

- Token Analytics baseline measurement (реальный тест без CMOS vs с CMOS).
- Документация setup для Claude Desktop/Code.
- GUI: визуальная проверка на реальных данных (UI рендерится, но данных пока нет).
- Push на GitHub (много uncommitted changes).

## Где мы в roadmap

- **Documentation phase:** 100% ✓
- **MVP Milestone 1 (Bootstrap):** 100% ✓
- **MVP Milestone 2 (Memory layers):** 100% ✓
- **MVP Milestone 3 (Two-LLM economy):** 100% ✓
- **MVP Milestone 4 (Vector index + MCP):** 100% ✓
- **MVP Milestone 5 (GUI skeleton):** 100% ✓
- **Token Analytics instrumentation:** 100% ✓
- **Portable build & launch:** 100% ✓
- **Integration hardening:** ~90%
- **MVP implementation overall:** ~93%.

## Ключевые контекстные факты

- GitHub repo: `https://github.com/dvgmdvgm/CMOS4LLM.git` — needs push (many uncommitted files).
- Owner использует Ollama с Gemma4 для Sub-LM задач.
- CI на Windows runner (GitHub Actions), Cargo caching enabled.
- 95 тестов: bootstrap (27), memory (30), retrieval (23), sub-lm (4), gateway (2), core (0), desktop (0).
- GUI: Tauri 2.x + React 19 + Zustand + Tailwind v4. In-process (без daemon).
- Token Analytics: SQLite-based, shared между MCP gateway и desktop app.
- MCP transport: newline-delimited JSON (не Content-Length framing).
- **Portable:** данные в `data/` в корне проекта (или `CMOS_DATA_DIR`). Никаких AppData/registry зависимостей.
- **WebView2 Runtime** — обязательная системная зависимость, батник ставит автоматически.

---

**Если только что открыл проект:** теперь читай [NEXT.md](./NEXT.md).
