# STATUS — где мы сейчас

> **Этот файл обновляется в конце каждой сессии работы над CMOS. 1 экран максимум.**
> Если STATUS не обновлён — следующая сессия начнётся со слепого понимания.

---

**Дата последнего обновления:** 2026-05-26
**Текущая фаза:** MVP implementation — scaffold complete (~5% MVP)
**Кто работал:** owner + Opus 4.6

---

## Что сделано в последнюю сессию

1. **Scaffold Cargo workspace** — `Cargo.toml` (workspace), 7 crates: `core`, `memory`, `sub-lm`, `gateway`, `policy`, `retrieval`, `cli`. Все компилируются.
2. **Tauri 2.x desktop app** — `apps/desktop/` с React 19 + TypeScript + Vite + pnpm. Окно открывается, показывает "CMOS Cognitive Console".
3. **CLI binary** — `cmos-cli hello` выводит версию и статус daemon.
4. **`run.bat` с auto-provisioning** — проверяет/ставит VS Build Tools, Rust, Node, pnpm, Tauri CLI, frontend deps. Один скрипт для чистой машины.
5. **Установлены VS Build Tools 2022** — были отсутствующие, ключевая зависимость для MSVC linker на Windows.
6. **Исправлен баг с em-dash в bat-файле** — non-ASCII символы ломали cmd.exe парсер.

## Что НЕ сделано (ждёт следующей сессии)

- CI pipeline (GitHub Actions).
- MVP Milestone 1: Bootstrap pipeline.
- Git commit scaffold'а (owner не дал команду).

## Где мы в roadmap

- **Documentation phase:** 100% ✓
- **MVP implementation:** ~5%. Scaffold готов, первый исполнимый код работает. Следующий шаг — CI или Bootstrap pipeline.

## Ключевые контекстные факты

- Owner часто переустанавливает систему / работает за разными машинами — поэтому `run.bat` с auto-provisioning критичен.
- Git for Windows кладёт свой `link.exe` в PATH, перехватывая MSVC linker — `run.bat` решает это через `vcvarsall.bat x64`.
- Bat-файлы на Windows не должны содержать non-ASCII символов (em-dash, кириллица в REM) — cmd.exe ломается.
- pnpm требует `pnpm approve-builds` для esbuild при первой установке.

---

**Если только что открыл проект:** теперь читай [NEXT.md](./NEXT.md).
