# 2026-05-26 — Portable Build & Launch

> Приложение сделано полностью портативным. run.bat доведён до one-click автоматизации.

---

## Контекст

MVP Milestones 1–5 были реализованы, Token Analytics инструментирован. GUI skeleton собирался, но не запускался — не хватало WebView2 Runtime, а данные хранились в AppData (не портативно). Owner хочет полную независимость от системы: после переустановки Windows — скопировал папку, запустил батник, всё работает.

---

## Что сделано

### Портативность данных
- `apps/desktop/src-tauri/src/main.rs` — `data_root` переключён с `dirs::data_local_dir()` (AppData) на portable логику:
  1. Проверяет env var `CMOS_DATA_DIR` (приоритет).
  2. Fallback: папка `data/` рядом с exe.
- `apps/desktop/src-tauri/Cargo.toml` — удалена зависимость `dirs = "6"`.
- `.gitignore` — добавлена запись `data/`.

### run.bat — автоматизация
- Добавлен `%USERPROFILE%\.cargo\bin` в PATH в начале скрипта (cargo часто не в PATH свежей сессии).
- Добавлена автоустановка WebView2 Runtime через Microsoft bootstrapper (winget ненадёжен — exit code 75).
- Frontend deps: `pnpm install --frozen-lockfile` → fallback на полный `pnpm install`. Убрана проверка "есть ли node_modules" — всегда синхронизирует.
- Release-режим теперь тоже запускает exe после сборки.
- `CMOS_DATA_DIR=%~dp0data` задаётся перед сборкой — данные всегда в корне проекта.

### WebView2 Runtime
- Установлен через Microsoft bootstrapper (`https://go.microsoft.com/fwlink/p/?LinkId=2124703`).
- Приложение запускается, окно "CMOS Cognitive Console" отображается.

---

## Ключевые решения

1. **Portable data через env var + exe-relative fallback.** Не используем AppData/registry для хранения данных. Причина: owner хочет полную независимость от системы. Данные живут в `data/` в корне проекта.

2. **WebView2 через bootstrapper, не winget.** Winget ненадёжен (exit code 75, проблемы с source). Microsoft bootstrapper (`/silent /install`) работает стабильно.

3. **Инструменты сборки остаются системными.** MSVC, Rust, Node.js, WebView2 требуют системной регистрации — их нельзя сделать portable. Но батник гарантирует автоустановку одним запуском. Это правильный trade-off: после переустановки Windows запускаешь батник один раз, он всё ставит.

4. **`pnpm install` всегда.** Вместо проверки "есть ли node_modules" — всегда запускаем `pnpm install --frozen-lockfile`. Если lockfile актуален — мгновенно. Если нет — полный install. Решает проблему "installed by a different package manager".

---

## Открытые вопросы

- Нет новых. Все существующие Q1–Q9 в ROADBLOCKS.md остаются актуальными.

---

## Следующий шаг

- Push всех uncommitted changes на GitHub.
- Token Analytics baseline measurement.
- Документация setup для Claude Desktop/Code.

См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы изменены:** `main.rs` (desktop), `Cargo.toml` (desktop), `run.bat`, `.gitignore`
- **Тесты:** 95 проходят, clippy clean, TypeScript clean
- **Результат:** MVP ~93%, приложение портативное и запускается
