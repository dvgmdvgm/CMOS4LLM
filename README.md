# CMOS — Cognitive Memory Operating System

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

**Documentation phase.** Архитектура согласована, реализация ещё не начата. См. STATUS.md.

---

## Что точно НЕ делает этот проект

См. [docs/03-scope/out-of-scope.md](./docs/03-scope/out-of-scope.md). Коротко: это не очередной RAG, не chat wrapper, не toy memory layer. Это операционная система для LLM-driven work.
