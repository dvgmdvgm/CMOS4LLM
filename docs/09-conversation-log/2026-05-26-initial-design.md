# 2026-05-26 — Initial design conversation

> Полная запись первой архитектурной беседы по CMOS. Это **главный** wake-up артефакт.
> Если ты вернулся к проекту через дни/недели и потерял контекст — прочитай этот файл, и ты восстановишь 90% reasoning'а за решениями.

---

## Контекст беседы

- **Owner** разрабатывает большой Django marketplace (~400K LoC) с использованием cloud LLM.
- На опыте этой работы owner осознал фундаментальные проблемы LLM: огромный расход токенов, отсутствие памяти между сессиями, drift проектных правил, невозможность долгосрочной консистентности.
- Owner поставил задачу: спроектировать систему, которая решает эти проблемы как **операционная система для LLM**, а не как очередной memory-wrapper.
- Беседа велась на русском с Opus 4.7. Технический язык docs выбран как гибрид: вижн/charter/conv-log на русском, technical spec/ADR на английском.

---

## Часть 1 — Фундаментальные проблемы LLM (Этапы 1–2 архитектурного исследования)

### Этап 1: Почему «больше контекста» не решает проблему

Зафиксированы 5 фундаментальных bottleneck'ов трансформерной архитектуры:

1. **Quadratic attention cost** — O(n²·d) compute и O(n²) KV-cache. Запрос с 1M контекстом стоит в ~15 000 раз больше FLOPs, чем 8K. Subquadratic архитектуры (Mamba, Ring Attention) жертвуют качеством recall.
2. **Lost-in-the-middle** — даже при advertised 1M контексте recall в середине падает до 30–50% от recall на краях. Это следствие positional encoding + training distribution.
3. **Effective context << Advertised context** — RULER benchmark: модели с advertised 128K имеют effective ~32K для сложного reasoning. 80–90% оплаченных токенов не работают.
4. **KV-cache не переносим между сессиями** — он привязан к весам, позициям, физической inference-ноде. Anthropic prompt caching (5min TTL), OpenAI/Gemini caching — попытки эксплуатировать KV внутри одной инстанции, но cross-session проблему не решают.
5. **Attention ≠ Memory** — attention это механизм взвешивания текущего контекста, не память. Если факт не в окне — модель его не знает. Все попытки добавить память (RAG, MemGPT, Letta) — внешние костыли поверх stateless inference.

**Доп. ограничения:**

- Текущая чат-парадигма имеет встроенную O(n²) стоимость по числу turns в долгой сессии (replay).
- RAG имеет 7 фундаментальных провалов: chunking теряет структуру, embedding ≠ semantic relevance, нет понимания времени, нет графовых связей, top-K замаскированная политика, нет task-aware retrieval, reranker — пластырь.
- Vector embeddings — lossy compression (768–4096-мерный вектор не отражает все нюансы текста). Хороши для fuzzy similarity, плохи для exact, relational, temporal, constraint queries.
- Hallucinated inconsistency возникает из-за: (а) правила не в контексте, (б) правило в контексте без veto power, (в) правило в форме прозы, не активирующей constraint satisfaction в attention.

**Главный вывод этапа 1:** проблемы LLM — не bugs следующих моделей, а архитектурные следствия трансформера. **Их нельзя устранить улучшением модели; их можно только обойти внешним слоем.**

### Этап 2: Существующие решения и где они ломаются

5 кластеров существующих решений, каждый решает 20% задачи:

- **Retrieval-based (RAG, GraphRAG, HippoRAG, ColBERT):** хороши для fuzzy similarity, плохи для policy enforcement, drift, observability.
- **Conversational/Agent memory (MemGPT, Letta, Zep, Mem0):** ориентированы на personal assistant memory, не project brain. LLM как memory manager — дорого и unreliable.
- **Native vendor memory (Claude Projects, ChatGPT Memory, Cursor, Windsurf):** static knowledge bases, нет автоматической экстракции, нет drift detection, нет cross-project memory.
- **System-level (MCP, DSPy):** transport layer и context engineering, но не определяют memory semantics.
- **Frontier (RMT, RETRO, Mamba, KV-distillation, neurosymbolic):** research или не deployed; вдохновение для архитектуры, но не готовое решение.

**Никто не строит memory как OS-level concept. Никто не делает policy enforcement first-class. Observability — ноль.**

---

## Часть 2 — Архитектура CMOS (Этапы 3–6)

### Этап 3: Высокоуровневая архитектура

**Ключевая идея:** CMOS — внешний когнитивный субстрат. LLM превращается в stateless inference engine `f(context) → tokens`. Source of truth — CMOS, не модель.

**Компоненты:**

- **Gateway** (Rust/Go) — единая точка входа, MCP/HTTP/gRPC, multi-tenancy, sessions.
- **Context Hypervisor** — главный оркестратор. Получает query → классифицирует → планирует retrieval → собирает prompt с budget enforcement → отправляет в LLM → постпроцессит.
- **Retrieval Router** — комбинирует стратегии retrieval (symbol lookup, vector, graph traversal, temporal, episodic, hybrid, ColBERT). Не «одна стратегия», а параллельный план.
- **Memory Hierarchy L1–L5** (см. Этап 4).
- **Policy & Invariant Engine** — three-tier: suggestions / soft invariants / hard invariants. Symbolic, не embedded. Каждая политика — структурированный объект с scope, predicate, rationale, evidence_refs.
- **Constraint Solver** — constrained decoding для structured output, post-hoc validation для свободного кода.
- **Sub-LM Runtime** — пул локальных моделей (3B / 14B / 32B) для extraction, summarization, dedup, linting. Работают батчево и асинхронно в background.
- **Observability & Telemetry** — first-class, не afterthought.

### Этап 4: Memory Hierarchy

| Layer | Размер | TTL | Latency | Tech | Содержит |
|---|---|---|---|---|---|
| **L1** Working | 1K–16K | minutes | <1ms | RAM | assembled prompt + scratch |
| **L2** Session | 50K–500K | hours | <5ms | RocksDB | event log сессии: turns, decisions, scratch facts |
| **L3** Episodic | 1M–10M | days–weeks | <50ms | RocksDB + vector | задачи и их разборы; lessons; rejected approaches |
| **L4** Project | 100M–10B | indefinite | <100ms | Graph DB + vector + KV | онтология проекта, code symbols, policies, DNA |
| **L5** Archival | unlimited | indefinite (decay) | <1s | object storage | вся история, evolution, deprecated |

Promotion/demotion автоматические по эвристикам (access pattern + semantic importance + recency).

**Conflict resolution:** никогда не перезаписываем; новый факт vs существующий → ASK или auto-resolve через recency/version chain. Конфликты — first-class events.

**Никогда не hard-delete из L4/L5** — только tombstone, versioned.

### Этап 5: Radical Token Reduction (12 техник)

Ранжированы по impact:

| Техника | Экономия | Сложность |
|---|---|---|
| Sub-LM pre-filtering | 5–15× | Low |
| Symbolic pre-resolution | ∞ (где применимо) | Medium |
| Compressed cognition blocks | 5–20× (повторы) | High |
| Semantic delta encoding | 2–5× (внутри сессии) | Medium |
| Hierarchical summarization | 3–10× | Low |
| Lazy loading через references | 5–20× (code tasks) | Medium |
| Prompt caching awareness | 5–10× $$$ | Low |
| Policy injection in imperative form | 3–5× | Trivial |
| Differential retrieval | 1.5–3× | Low |
| Constraint hoisting | 1.5–2× | High |
| Cognitive replay skip | ∞ | Medium |
| Persistent KV-cache (frontier) | 10×+ | Very high |

**Composite realistic estimate:** 8–25× снижение облачных токенов на типичную задачу. 100× — best case, не baseline.

### Этап 6: Project DNA & Invariant Engine

Project DNA = constitution проекта (5K–20K токенов, всегда инъектируется):

1. Identity statement
2. Architectural pillars
3. Hard invariants (10–30 пунктов)
4. Style fingerprint
5. Forbidden patterns
6. Critical context

**Three-tier policy model:**

- Suggestions → mention в prompt.
- Soft invariants → mention + post-hoc warn.
- Hard invariants → constrained decoding (где применимо) + post-hoc check + repair loop / block.

**Drift Detection** в background через Sub-LM. Drift trends → suggested rules в DNA Editor.

**Invariant Evidence Graph:** каждое правило связано с evidence (decisions, incidents, PRs). Это immune system проекта.

---

## Часть 3 — Применимость к существующему проекту (Django marketplace)

Owner спросил: можно ли применить к существующему Django проекту 400K LoC.

**Ответ: да, и это главный практический use case.**

CMOS архитектурно проектируется как retrofittable subsystem. Не требует ни единого изменения в коде проекта. Bootstrap pipeline:

1. **Static analysis sweep** — AST-парсинг, извлечение моделей, views, urls, signals, middleware, settings, миграций. Без LLM.
2. **Schema & domain extraction** — из `models.py` строится доменная онтология (User, Product, Order, Cart, Payment...).
3. **Architectural pattern detection** — слои, middleware chain, signal flows, Celery, REST/GraphQL.
4. **Convention mining** — Sub-LM выводит де-факто правила: naming, размеры функций, paradigms, style.
5. **Git history mining** — log + blame + PR descriptions → temporal knowledge.
6. **Rejected approaches detection** — `# TODO: removed because`, deleted code in big refactors.
7. **Documentation ingestion** — README, docs/, ADRs, CHANGELOG, wiki через малую LLM.
8. **Interactive policy elicitation** — 20–50 вопросов owner'у → hard policy layer.

**Реалистичные ожидания для V1 (6–9 мес работы CMOS):**

- Снижение токенов на типичную задачу: 5–15×.
- Стабильность правил между сессиями: ~30% → ~85%+.
- Время онбординга на новую фичу: с минут перечитывания → секунды injection.
- Понимание «почему так сделано»: сильное при богатом git history.

**Что не получится:**

- Магически понять недокументированную бизнес-логику без elicitation.
- Помнить устные решения, принятые до подключения.
- Полностью заменить ручной code review.

---

## Часть 4 — GUI design (Cognitive Console)

### 6 ключевых GUI-решений (зафиксированы owner'ом)

1. **Density:** высокая (Datadog/DevTools/Wireshark-style). Цель — полезность, не красивость.
2. **Где живёт UI:** гибрид-shell — Tauri native shell + web-core + IDE plugins, все смотрят в один daemon.
3. **Real-time:** WebSocket для живых view (live inference, drift events, token meter); snapshot-driven для исторических.
4. **Целевой пользователь:** на старте — owner один; архитектурно закладывается multi-user; manager view — позже.
5. **Time Travel Debugging:** в V1 (по решению owner'а). Это диктует event-sourced хранилище inferences.
6. **Counterfactual mode:** реализуем (решение owner'а). Архитектура должна позволять воспроизводить inference с изменёнными inputs.
7. **Multi-project из коробки.**

### 9 экранов + 1 always-on overlay

1. **Dashboard** — health-check проекта в одном экране: token economy, memory health, drift monitor, recent episodes, active policies.
2. **Live Inference Inspector** — флагман. Real-time показывает: что собрано в context, что исключено, cache hit ratio, streaming response, post-gen validation. Кнопки replay/retry-without-X/save-as-episode/explain.
3. **Memory Browser** — file manager для памяти. Layers tree, items list, detail panel с evidence, usage stats, history.
4. **Knowledge Graph Viewer** — Cytoscape, режимы Domain / Code symbols / Decisions evolution / Module dependencies / Combined.
5. **Project DNA Editor** — versioned, diff viewer, evidence ссылки, suggested rules from drift detection. Token budget визуализирован.
6. **Drift Monitor** — timeline violations по категориям, open violations с действиями (fix/exempt/snooze), trends.
7. **Token Analytics** — headline (cloud used / saved / reduction ratio), savings breakdown по техникам, daily timeseries vs simulated baseline, top cost queries.
8. **Episodes Browser** — Linear/Jira-like, но с фокусом на reasoning. Каждый episode = case study с lessons.
9. **Policy Manager** — bulk operations, A/B testing групп правил.
10. **Cognitive Trace overlay** — always-on, нижний правый угол. Recent activity, текущее состояние, кнопки open/pause.

### Killer features

- **Time Travel Debugging:** открыл прошлый turn → видишь полный assembled context того момента → воспроизводишь / меняешь один item / counterfactual.
- **Counterfactual mode:** Sub-LM в background перепрогоняет последние N inferences с альтернативным набором политик. A/B testing вместо «молитвы».
- **Memory Heatmap:** карта памяти, окрашенная по частоте использования. Видно мёртвый груз и горячие 20%.

### Стек GUI

- Shell: **Tauri 2.x** (малый binary, Rust backend = тот же язык, что core).
- Frontend: **React + TypeScript**.
- State: **Zustand** + WebSocket.
- Charts: **uPlot** (timeseries без лагов) + **Recharts** (общие).
- Graph viz: **Cytoscape.js** (для больших графов).
- Code viewer: **Monaco** (тот же, что VSCode).
- Theme: dark, dense, monospace headings.

---

## Часть 5 — Wake-up resilience и документация

Owner явно сказал: «я могу лечь поспать, проснуться через пару дней и продолжить с того же места». Это **главный non-functional requirement** проекта.

Документация спроектирована вокруг этого:

```
cmos/
├── README.md                       <- entry point
├── STATUS.md                       <- ⭐ где мы сейчас (1 экран)
├── NEXT.md                         <- ⭐ следующие шаги (1 экран)
├── ROADMAP.md                      <- MVP / V1 / V2 / V3 / Future
├── ROADBLOCKS.md                   <- открытые вопросы
├── CLAUDE.md                       <- инструкции для Claude
└── docs/
    ├── 00-charter.md
    ├── 01-architecture.md
    ├── 02-decisions/               <- ADR
    ├── 03-scope/                   <- mvp/v1/v2/v3/future/out-of-scope
    ├── 04-components/              <- per-component spec
    ├── 05-gui/                     <- 9 экранов + overlay
    ├── 06-bootstrap/               <- onboarding existing projects
    ├── 07-research/
    ├── 08-glossary.md
    └── 09-conversation-log/        <- хронология (этот файл здесь)
```

**Hard ритуал в конце каждой сессии:**

1. Обновить STATUS.md.
2. Обновить NEXT.md.
3. Создать новый файл в `docs/09-conversation-log/`.
4. Если есть архитектурное решение → новый ADR.
5. Если открытый вопрос → запись в ROADBLOCKS.md.

---

## Что было решено в этой сессии (квинтэссенция)

1. CMOS — внешний слой над stateless LLM. Source of truth — CMOS, LLM — co-processor.
2. Two-LLM economy: heavy work → Sub-LM локально; cloud LLM получает сжатый отфильтрованный контекст.
3. 5-уровневая memory hierarchy с явными свойствами per-layer и promotion/demotion.
4. Three-tier policies с post-hoc enforcement.
5. Project DNA как constitution.
6. Bootstrap для существующих проектов — first-class flow (это как owner будет применять CMOS на своём marketplace).
7. GUI: 9 экранов + Cognitive Trace overlay, плотная DevTools-style плотность, Tauri hybrid shell.
8. Time Travel Debugging и Counterfactual mode — в V1.
9. Multi-project из коробки.
10. Documentation phase first; hard ритуал wake-up resilience.

## Что осталось обсудить (open questions)

См. [ROADBLOCKS.md](../../ROADBLOCKS.md). Главные:

- Q1: Какая локальная модель для Sub-LM на Windows-машине owner'а (зависит от GPU/RAM).
- Q3: Privacy-режим (Sub-LM редактирует чувствительные секции перед отправкой в cloud).
- Q4: Multi-user в одном проекте (V2).
- Q7: Как тестировать сам CMOS.
- Q8: Лицензия (open-source vs proprietary).

## Что делать в следующей сессии

См. [NEXT.md](../../NEXT.md). Главное:

1. Завершить documentation phase (architecture.md, scope, components, GUI specs, bootstrap).
2. Technical Decision Spike → серия ADR-011…ADR-016 по выбору стека.
3. Заморозить MVP scope.
4. Стартовый репозиторий + CI.
5. MVP Milestone 1: Bootstrap pipeline для Django marketplace (первый исполнимый код).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.7
- **Длительность беседы:** многоходовая, через несколько частей (Part 1, Part 2, GUI discussion, документация)
- **Результат:** скелет проекта в `D:\AI Projects\CMOS\` с anti-forgetting документацией; 10 ADR проектируются; documentation phase активна.
