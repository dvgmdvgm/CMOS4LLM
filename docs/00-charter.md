# CMOS Charter — vision, scope, non-goals

> Этот документ — **корень всего проекта**. Любая последующая работа должна быть проверяема против него.
> Если решение противоречит charter — либо charter обновляется (через ADR), либо решение отменяется.

---

## Кто и зачем

**Owner** — разработчик большого Django marketplace (~400K LoC). Каждый день работает с cloud LLM при разработке этого marketplace.

В процессе owner осознал: текущие LLM-инструменты в принципе не способны:

- Помнить, что в проекте `border-radius` всегда `4px`.
- Помнить, что money-поля — всегда `Decimal`.
- Помнить, что webhook'и должны быть idempotent (после инцидента 2025-04-12 с двойным списанием).
- Помнить, что в 2024 был отвергнут PayPal в пользу Stripe — и ПОЧЕМУ.
- Не пересчитывать половину чата при каждом следующем turn.
- Не забывать архитектурные решения через неделю.
- Не нарушать правила, которые они только что согласовали.

Эти проблемы — **не bugs следующих моделей**. Это архитектурные следствия трансформера + stateless API. Их нельзя решить «просто увеличить контекст до 10M». Их можно решить только **внешним слоем**, который берёт на себя память, идентичность и политики.

**CMOS — это и есть такой слой.**

---

## Vision (одна страница)

CMOS — это **внешний когнитивный субстрат для LLM-driven разработки**. Операционная система, в которой LLM — это co-processor (как CPU/GPU), а память, идентичность проекта, политики, история решений, наблюдаемость — это OS-level первичные сущности.

В мире с CMOS:

- LLM получает не «весь чат заново», а **сжатый, релевантный, валидированный** контекст.
- Проект помнит свои правила между сессиями, между LLM-провайдерами, между моделями.
- Каждое архитектурное решение фиксируется с rationale и evidence; через год можно открыть и понять, почему так.
- Drift проектных правил детектится автоматически; нарушения видны сразу, не через месяц на review.
- Owner видит, **что именно** попало в каждый prompt, **почему** оно попало, и **сколько токенов** на этом сэкономлено.
- Любой прошлый inference можно time-travel-debug'нуть: открыть, переиграть, попробовать counterfactual.
- Один проект ↔ много проектов из коробки; проекты не путаются.
- Cloud LLM работает в режиме «вызывают, когда нужно»; всю чёрную работу (extraction, summarization, dedup, linting, drift detection) делает локальный Sub-LM на машине owner'а.

**Цель в одной строке:** снизить расход облачных токенов в 5–25× на типичных задачах, дать проекту постоянную память, сделать процесс LLM-разработки наблюдаемым.

---

## Что входит в scope

1. **Memory infrastructure** — 5-уровневая иерархия (L1 working / L2 session / L3 episodic / L4 project / L5 archival).
2. **Context orchestration** — Cognitive Hypervisor: классификация задачи, планирование retrieval, budget-aware assembly, attention-aware prompt rendering.
3. **Two-LLM economy** — local Sub-LM для bulk cognitive work, cloud LLM для критического reasoning.
4. **Project DNA & Invariant Engine** — three-tier policies (suggestion / soft / hard), versioned DNA, drift detection, evidence-tracked invariants.
5. **Bootstrap pipeline** для существующих проектов — Django, Node/TS (V1+), generic.
6. **Cognitive Console GUI** — 9 экранов + Cognitive Trace overlay; Tauri hybrid shell.
7. **MCP integration** + IDE plugins (VS Code в MVP/V1, JetBrains в V2).
8. **Multi-project** из коробки.
9. **Observability** как first-class — telemetry, cognitive traces, token analytics, baseline comparison.
10. **Wake-up resilience documentation** — сам процесс разработки CMOS должен переживать паузы owner'а в дни/недели.

---

## Non-goals (что CMOS НЕ делает)

- **CMOS не модифицирует модель.** Никаких fine-tuning, никаких custom inference. Только внешний слой над off-the-shelf LLM API + локальные open-weight модели для Sub-LM.
- **CMOS не пишет код вместо тебя.** Это инфраструктура, через которую ты пишешь код с LLM. Code generation — задача LLM, CMOS — его контекст.
- **CMOS не заменяет документацию проекта.** Он дополняет и обогащает её, но READMEs, ADRs, design docs — by humans, for humans.
- **CMOS не заменяет code review.** Drift detection и invariant enforcement — это immune system, но не review. Reviewer — человек.
- **CMOS не auto-pilot для рефакторинга.** Counterfactual mode — это инструмент анализа, не автоматический оптимизатор.
- **CMOS не competitor MCP/Claude/Cursor.** Это слой **под** ними. Они — клиенты CMOS.
- **CMOS не general-purpose chat memory.** Он спроектирован для project-aware development. Chat-with-history — побочный эффект, не цель.
- **CMOS не toy / proof-of-concept.** Целью является production-grade infrastructure, использовать которую owner будет на своём marketplace в реальной работе.

---

## Принципы дизайна

1. **Stateless LLM, stateful CMOS.** LLM никогда не source of truth. CMOS — единая память.
2. **Two-LLM economy.** Heavy work — локально, cloud — только финальное reasoning. Не отправляй raw chat в cloud.
3. **Symbolic where possible, neural where needed.** Если задачу можно решить детерминированно (graph lookup, AST validation) — не зови LLM.
4. **Append-only memory.** Никогда не hard-delete. Только tombstone, versioning, supersession.
5. **Evidence-tracked invariants.** Каждое правило связано с decisions/incidents/PRs. Никаких rules from nowhere.
6. **Attention-aware prompts.** Critical в начало и конец, middle — менее критичное. Императивы вместо прозы.
7. **Budget-bound.** Каждый assembly ограничен токеновым budget'ом; меньше = качественнее attention.
8. **Observable by default.** Каждая retrieval-выборка, каждое решение Hypervisor'а, каждый drift event — видны в Cognitive Console.
9. **Multi-project equality.** Никогда не assume один проект.
10. **Wake-up resilient documentation.** Документация устроена так, что любое окно/контекст восстанавливает картину за 5 минут.

---

## Реалистичные ожидания

**Что мы обещаем:**

- 5–25× снижение облачных токенов на типичных задачах (после Bootstrap + 1–2 недель работы CMOS на проекте).
- Качественный скачок в стабильности правил проекта между сессиями (с ~30% соблюдения до ~85%+).
- Сильное понимание эволюции проекта — при условии богатого git history и/или регулярного использования CMOS.

**Что мы НЕ обещаем:**

- 100× снижение токенов как baseline. Возможно для best cases (повторяющиеся паттерны, read-heavy queries), не как ожидание.
- Магическое понимание недокументированной бизнес-логики без elicitation.
- Полную замену human review.
- Помнить решения, принятые устно/в Slack до подключения CMOS (можно загрузить вручную).

**Любая публичная цифра должна быть прослеживаема к воспроизводимому бенчмарку.** См. [CLAUDE.md](../CLAUDE.md) → Benchmark honesty.

---

## Кто пользователи

**Уровень 1 (MVP):** owner-один. Solo разработчик с проектом ~100K–1M LoC. Использует CMOS на своей машине, через MCP-клиент (Claude Code / Cursor / etc.) или напрямую через API.

**Уровень 2 (V2):** small team (2–10 разработчиков) с shared CMOS instance. Multi-user, attribution, comments.

**Уровень 3 (Future):** organization / SaaS / cross-team. Federated memory, marketplace для project DNA templates.

В этом charter и в ROADMAP **по умолчанию мы проектируем для уровня 1**, но архитектурно закладываем уровень 2 (multi-project из коробки, изолированные namespaces, audit trail).

---

## Условия успеха проекта

**MVP считается успешным**, если:

1. Bootstrap pipeline за разумное время (часы) обрабатывает Django marketplace owner'а и строит L4.
2. Owner может работать через CMOS на marketplace продуктивно — то есть не медленнее и без потери возможностей по сравнению с прямым использованием Claude/Cursor.
3. Token Analytics показывает измеримое снижение облачных токенов хотя бы в 3× на типичной задаче (это нижняя граница; целевые 5–25× — для V1).
4. Memory survives session boundaries — owner закрыл клиент, открыл через 3 дня, контекст marketplace сохранён.
5. GUI Live Inference Inspector работает и показывает, что именно попало в каждый prompt.

**V1 считается успешным**, если:

1. Все 9 экранов GUI функциональны.
2. Time Travel Debugging работает.
3. Counterfactual mode работает.
4. Drift Monitor показывает реальные нарушения и предлагает rule promotions.
5. CMOS используется owner'ом ежедневно на marketplace без обхода.
6. Документация проекта в состоянии, в котором стороннему senior-engineer'у можно дать `D:\AI Projects\CMOS\` и через час он понимает архитектуру.

---

## Связанные документы

- [README.md](../README.md) — entry point.
- [ROADMAP.md](../ROADMAP.md) — MVP / V1 / V2 / V3 / Future.
- [docs/01-architecture.md](./01-architecture.md) — техническая архитектура.
- [docs/02-decisions/](./02-decisions/) — все ADR.
- [docs/09-conversation-log/](./09-conversation-log/) — хронология решений.
