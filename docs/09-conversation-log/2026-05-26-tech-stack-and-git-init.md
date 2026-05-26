# 2026-05-26 — Tech stack ADRs and git init

> Финальная сессия documentation phase. Выбор стека зафиксирован, репозиторий инициализирован.

---

## Контекст сессии

- **Owner** дал команду: «допиши документацию до конца и выполни git init».
- Задача: написать ADR-011..016 (выбор стека), инициализировать git, сделать initial commit.

---

## Что сделано

### ADR-011..016 — выбор технологического стека

Проведено исследование кандидатов по каждой категории (web search, crates.io, GitHub). Ключевые находки:

- **Graph DB:** Kuzu архивирован (2025), CozoDB заброшен (последний релиз Dec 2023). Embedded graph DB экосистема в Rust мертва.
- **Vector Index:** LanceDB (native Rust, 109K downloads/month) и USearch (87K downloads/month) — два сильных варианта.
- **Sub-LM:** llama-cpp-2 — явный лидер (111K downloads/month, CUDA, LoRA, GGUF).
- **Event Storage:** SQLite (rusqlite, 7.5M downloads/month) vs redb (783K, pure Rust) vs RocksDB (1.8M, но тяжёлый build).

Owner подтвердил рекомендации по всем 4 категориям:

| ADR | Решение | Почему |
|-----|---------|--------|
| ADR-011 | Rust | Единый язык с Tauri, zero-cost, llama-cpp-2 bindings |
| ADR-012 | SQLite + recursive CTEs | Все graph DB мертвы; SQLite надёжен, SQL для time-travel |
| ADR-013 | LanceDB | Batteries-included: versioning, hybrid search, native Rust |
| ADR-014 | llama-cpp-2 | In-process, GGUF, CUDA, LoRA, активно поддерживается |
| ADR-015 | Tauri 2.x + React + TS | Один web core для desktop/VS Code/web, богатая экосистема |
| ADR-016 | SQLite WAL | Unified engine с L4, SQL для temporal queries, zero deps |

### Git init

- Создан `.gitignore` (Rust, Node, Tauri, SQLite, Lance, models, secrets).
- Создан `LICENSE` (MIT — owner's choice, закрывает ROADBLOCKS Q8).
- `git init` + initial commit: 66 files, 4769 insertions.

---

## Ключевые решения

1. **SQLite как unified storage engine** — L2, L3, L4 все используют SQLite (разные файлы). Это упрощает backup, debugging, и снижает количество зависимостей.
2. **Нет viable embedded graph DB в Rust (2026)** — это неожиданная находка. Kuzu и CozoDB оба мертвы. Решение: SQLite + recursive CTEs — boring but works.
3. **LanceDB over USearch** — owner предпочёл batteries-included подход (versioning, hybrid search) над lightweight index.
4. **MIT лицензия** — закрывает Q8 из ROADBLOCKS.

---

## Что дальше

Documentation phase **завершена на 100%**. Следующая сессия — **первый код**:
1. Scaffold Cargo workspace + Tauri app.
2. CI pipeline.
3. MVP M1: Bootstrap pipeline.

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Результат:** documentation phase complete. Git repo initialized. Ready for implementation.
