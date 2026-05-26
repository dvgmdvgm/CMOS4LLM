# 2026-05-26 — Scaffold Cargo workspace + Tauri app

> Первая сессия имплементации. Создан рабочий scaffold проекта с auto-provisioning.

---

## Контекст

Documentation phase завершена на 100% в предыдущих сессиях (ADR-001..016, architecture, components, GUI specs, bootstrap spec). Git repo инициализирован с initial commit (66 файлов документации). Стек зафиксирован: Rust, SQLite, LanceDB, llama-cpp-2, Tauri 2.x + React + TS.

Задача этой сессии: создать scaffold Cargo workspace + Tauri app — первый исполнимый код.

---

## Что сделано

### Cargo workspace
- `Cargo.toml` — workspace root, edition 2024, 8 members.
- `crates/core/` — `cmos_core::version()`, базовый crate.
- `crates/memory/`, `crates/sub-lm/`, `crates/gateway/`, `crates/policy/`, `crates/retrieval/` — stub lib.rs с `init()`.
- `crates/cli/` — binary `cmos-cli` с clap, команда `hello`.
- `.cargo/config.toml` — Windows stack size настройка.

### Tauri desktop app
- `apps/desktop/package.json` — React 19, @tauri-apps/api 2, Vite 6, TypeScript 5.9.
- `apps/desktop/src/App.tsx` — минимальный UI, вызывает Tauri command `get_version`.
- `apps/desktop/src-tauri/` — Tauri 2.x backend, зависит от `cmos-core`.
- `apps/desktop/src-tauri/tauri.conf.json` — конфигурация окна 1280x800.
- Placeholder иконки (32x32, 128x128, ico).

### run.bat (auto-provisioning)
- Проверяет и устанавливает: VS Build Tools 2022, Rust, Node.js, pnpm, Tauri CLI.
- Устанавливает frontend deps (`pnpm install`).
- Три режима: dev (hot reload), debug, release.
- Загружает MSVC environment через `vcvarsall.bat x64` — решает конфликт с GNU `link.exe` из Git for Windows.

---

## Ключевые решения

1. **Subroutine `:find_vcvars` вместо `for` цикла** — `for %%d in (...)` с путями, содержащими `Program Files (x86)` (скобки!), ломает cmd.exe парсер. Subroutine с `if exist` — надёжнее.

2. **Никаких non-ASCII в bat-файлах** — em-dash (`—`) в REM-строках вызывает мгновенное закрытие консоли. Cmd.exe не умеет парсить UTF-8 в bat-файлах. Заменено на ASCII `--`.

3. **`vcvarsall.bat x64` обязателен перед cargo build** — без него Rust находит GNU `link.exe` из `D:\PORTABLE\DevStack\git\usr\bin\` вместо MSVC linker. Это специфика portable Git на Windows.

4. **Path dependency `../../../crates/core`** — из `apps/desktop/src-tauri/` нужно три уровня вверх до workspace root, не два. Ошибка в первой версии давала `apps/crates/core` (несуществующий путь).

5. **pnpm approve-builds** — pnpm 11.x по умолчанию блокирует postinstall scripts (esbuild). Нужен явный approve. В `run.bat` это пока не автоматизировано — при первом запуске может потребоваться ручной `pnpm approve-builds esbuild`.

---

## Открытые вопросы

- **pnpm approve-builds** не автоматизирован в `run.bat` — при первом запуске на чистой машине может потребоваться ручное действие. Не критично, но стоит добавить.
- **Git commit** не сделан — owner не дал команду. Scaffold не закоммичен.

---

## Следующий шаг

Git commit scaffold'а → CI pipeline или сразу MVP M1 (Bootstrap pipeline). См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Результат:** scaffold готов, компилируется, Tauri app запускается. Первый исполнимый код CMOS.
