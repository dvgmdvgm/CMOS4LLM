# STATUS — где мы сейчас

> **Этот файл обновляется в конце каждой сессии работы над CMOS. 1 экран максимум.**
> Если STATUS не обновлён — следующая сессия начнётся со слепого понимания.

---

**Дата последнего обновления:** 2026-05-26
**Текущая фаза:** Documentation phase — 100% COMPLETE. Ready for implementation.
**Кто работал:** owner + Opus 4.6

---

## Что сделано в последнюю сессию

1. **Написаны ADR-011..016** — выбор стека:
   - ADR-011: Rust как язык core daemon.
   - ADR-012: SQLite + recursive CTEs как graph store для L4.
   - ADR-013: LanceDB как vector index для L3/L4.
   - ADR-014: llama-cpp-2 как Sub-LM runtime.
   - ADR-015: Tauri 2.x + React + TypeScript для GUI.
   - ADR-016: SQLite WAL как event storage для L2/L3.
2. **Git init выполнен** — initial commit (66 files, 4769 lines), лицензия MIT.
3. **Закрыт ROADBLOCKS Q8** — лицензия MIT.

## Что НЕ сделано (ждёт следующей сессии)

- Стартовая структура исходников (Cargo workspace, Tauri app scaffold).
- CI pipeline (GitHub Actions).
- MVP Milestone 1: Bootstrap pipeline (первый исполнимый код).

## Где мы в roadmap

- **Documentation phase:** 100% ✓ (charter, ADR-001..016, scope, glossary, architecture, components, GUI, bootstrap, research — всё готово).
- **MVP implementation:** не начато. Следующий шаг — scaffold Cargo workspace + Tauri app.

## Ключевые контекстные факты

- Owner — разработчик Django-marketplace ~400K LoC. CMOS будет применяться к этому проекту первым.
- Стек зафиксирован: Rust, SQLite (graph + events), LanceDB (vectors), llama-cpp-2 (Sub-LM), Tauri 2.x + React + TS (GUI).
- Git repo инициализирован, initial commit сделан. Лицензия MIT.
- Wake-up resilience — главное правило: документация спроектирована так, что любое окно чата восстанавливает контекст за 5 минут.

---

**Если только что открыл проект:** теперь читай [NEXT.md](./NEXT.md).
