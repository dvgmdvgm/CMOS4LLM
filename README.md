# CMOS — Cognitive Memory Operating System

[![CI](https://github.com/dvgmdvgm/CMOS4LLM/actions/workflows/ci.yml/badge.svg)](https://github.com/dvgmdvgm/CMOS4LLM/actions/workflows/ci.yml)

**Внешний когнитивный субстрат для LLM.** Слой, который берёт на себя память, идентичность проекта, политики и наблюдаемость, превращая LLM из «собеседника, который всё помнит» в stateless inference engine — чистую функцию `f(context) → tokens`.

Цели в одной строке: **снизить расход облачных токенов в 5–25× на типичных задачах**, дать проекту **постоянную память между сессиями**, и сделать процесс работы LLM **наблюдаемым** через GUI.

---

## ⚡ Если ты только что открыл этот проект (после паузы / в новом окне чата)

Прочитай в этом порядке (5 минут):

1. **[STATUS.md](./STATUS.md)** — где мы сейчас, что сделано в последнюю сессию.
2. **[NEXT.md](./NEXT.md)** — конкретные следующие 3–5 шагов.
3. **Последний файл в [docs/09-conversation-log/](./docs/09-conversation-log/)** — контекст последней встречи с подробностями.

После этих трёх файлов у тебя есть полная картина «что, где и зачем» и можно работать.

Если нужны детали глубже:

- **[ROADMAP.md](./ROADMAP.md)** — границы MVP / V2 / V3 / Future.
- **[ROADBLOCKS.md](./ROADBLOCKS.md)** — открытые вопросы, ждут решения.
- **[docs/00-charter.md](./docs/00-charter.md)** — vision проекта, scope, non-goals.
- **[docs/01-architecture.md](./docs/01-architecture.md)** — полная архитектура (TODO).
- **[docs/02-decisions/](./docs/02-decisions/)** — все ADR с rationale.
- **[CLAUDE.md](./CLAUDE.md)** — инструкции для будущего Claude в этом проекте.

---

## Wake-up resilience: главное правило

В конце **каждой** сессии работы над CMOS обязательно:

1. Обновлён `STATUS.md` (где мы сейчас).
2. Обновлён `NEXT.md` (что дальше).
3. Создан новый файл в `docs/09-conversation-log/YYYY-MM-DD-<topic>.md` с ключевыми решениями.
4. Если приняты архитектурные решения — добавлен/обновлён ADR в `docs/02-decisions/`.
5. Если обнаружен открытый вопрос — записан в `ROADBLOCKS.md`.

Это hard-ритуал. Без него начинать новую сессию = строить замок на песке.

---

## Структура документации

```
cmos/
├── README.md                       <- ты здесь
├── STATUS.md                       <- ⭐ текущее состояние (1 экран)
├── NEXT.md                         <- ⭐ ближайшие действия (1 экран)
├── ROADMAP.md                      <- MVP / V2 / V3 / Future
├── ROADBLOCKS.md                   <- открытые вопросы
├── CLAUDE.md                       <- инструкции для Claude
└── docs/
    ├── 00-charter.md               <- vision, scope, non-goals
    ├── 01-architecture.md          <- полная архитектура (TODO)
    ├── 02-decisions/               <- ADR-style записи решений
    ├── 03-scope/                   <- mvp/v2/v3/future/out-of-scope
    ├── 04-components/              <- spec на каждый компонент
    ├── 05-gui/                     <- 9 экранов GUI + overlay
    ├── 06-bootstrap/               <- onboarding существующих проектов
    │   └── django-marketplace.md   <- основной use case (Django marketplace)
    ├── 07-research/                <- frontier, открытые направления
    ├── 08-glossary.md              <- термины
    └── 09-conversation-log/        <- хронология решений
```

---

## Текущая фаза

**MVP implementation — ~80% done.** Milestones 1–4 закрыты (bootstrap, memory layers, two-LLM economy, vector index + MCP hybrid assembly). 95 тестов проходят. См. STATUS.md.

---

## Quick Start — как проверить, что CMOS работает

### Предварительные требования

- Rust toolchain (rustup + cargo)
- Ollama (для embedding-модели, опционально для базового теста)

### 1. Собрать проект

```bash
cargo build --workspace
```

### 2. Запустить тесты

```bash
cargo test --workspace
```

Ожидаемый результат: 95 тестов проходят, включая 2 end-to-end MCP теста.

### 3. Проверить MCP server вручную

Запустить сервер:
```bash
cargo run -p cmos-cli -- mcp --root ./test-project
```

Сервер слушает stdin/stdout (JSON-RPC, newline-delimited JSON). Отправить initialize:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"manual-test","version":"0.1.0"}}}
```

Ожидаемый ответ: JSON с `serverInfo.name = "cmos"` и 6 tools в capabilities.

### 4. Подключить к Claude Desktop / Claude Code

Добавить в конфиг MCP серверов (Claude Desktop: `claude_desktop_config.json`, Claude Code: `.claude/settings.json`):

```json
{
  "mcpServers": {
    "cmos": {
      "command": "path/to/cmos-cli.exe",
      "args": ["mcp", "--root", "path/to/your/project"]
    }
  }
}
```

После подключения Claude получит доступ к 6 инструментам:
- `cmos_write_memory` — записать факт в L1 (рабочая память)
- `cmos_read_memory` — прочитать слот из L1
- `cmos_query_memory` — запросить L2/L3 эпизоды или L4 факты
- `cmos_assemble_context` — собрать оптимизированный контекст для задачи
- `cmos_search_similar` — семантический поиск (требует Ollama)
- `cmos_memory_stats` — статистика по всем уровням памяти

### 5. Bootstrap проекта (заполнить память)

```bash
cargo run -p cmos-cli -- bootstrap --root path/to/your/project --project-id my-project
```

Это сканирует git history, структуру файлов, и заполняет L2/L3/L4 память.

### 6. Проверить, что память работает

После bootstrap:
```bash
cargo run -p cmos-cli -- query --root path/to/your/project --project-id my-project --layer L4
```

Должны появиться извлечённые факты о проекте (conventions, decisions, policies).

---

## Что точно НЕ делает этот проект

См. [docs/03-scope/out-of-scope.md](./docs/03-scope/out-of-scope.md). Коротко: это не очередной RAG, не chat wrapper, не toy memory layer. Это операционная система для LLM-driven work.
