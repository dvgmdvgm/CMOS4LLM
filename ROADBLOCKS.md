# ROADBLOCKS — открытые вопросы

> Всё, что **не решено** и блокирует имплементацию. Каждый вопрос имеет статус и owner.
> Когда вопрос закрыт — он переезжает в соответствующий ADR в `docs/02-decisions/` и удаляется отсюда.

---

## Активные вопросы

### Q1: Какая локальная модель для Sub-LM на Windows-машине owner'а?

- **Контекст:** owner работает на Windows 11 Pro. Нужен local inference. Качество vs скорость vs RAM.
- **Кандидаты:** Qwen 2.5 Coder 14B, Phi-4, Llama 3.3 70B (только если есть VRAM), Qwen 2.5 32B.
- **Зависит от:** характеристики GPU/RAM owner'а (нужно уточнить).
- **Решается в:** ADR-014 после technical spike.
- **Owner:** owner + Claude.

### Q2: Какой precise бюджет токенов default'ом для context assembly?

- **Контекст:** В архитектуре сказано «16K–64K» как anti-bloat. Нужно конкретное число для каждого class задач.
- **Зависит от:** эксперименты на Django marketplace.
- **Решается в:** после первых benchmark'ов на реальных задачах. До этого — MVP использует 32K hard.

### Q3: Что делать с приватностью кода?

- **Контекст:** Cloud LLM получает контекст из репозитория. Для Django marketplace это **production codebase с PII / payment logic**.
- **Кандидаты:**
  - (a) Разрешить full code → cloud (current default; быстро).
  - (b) Sub-LM редактирует/анонимизирует чувствительные секции перед отправкой (сложнее, но правильнее для production).
  - (c) Local-only mode (только Sub-LM, без cloud) — для регулируемых сценариев.
- **Зависит от:** требований owner'а к compliance.
- **Решается в:** ADR-017 (TBD).

### Q4: Кто owns DNA — один человек или команда?

- **Контекст:** В V1 owner один. Но архитектурно надо знать заранее, чтобы не переделывать.
- **Сейчас зафиксировано (ADR-004):** Multi-project из коробки, но не сказано про multi-user в одном проекте.
- **Решается в:** новый ADR, когда станет актуально (вероятно V2).

### Q5: Как разрешать конфликты между новой версией DNA и существующими политиками?

- **Контекст:** При обновлении DNA старые политики могут противоречить новой constitution.
- **Кандидаты:**
  - Auto-deprecate с человеческим review.
  - Block на конфликте.
  - Версионирование с явным choice "use vX of DNA для этой сессии".
- **Решается в:** ADR-018 (TBD).

### Q6: Bootstrap для проектов без git history?

- **Контекст:** Django marketplace owner'а с git'ом есть, но архитектура должна работать и без.
- **Решение в MVP:** документация только для git-aware bootstrap. Поддержка без git — V1+.

### Q7: Как тестировать сам CMOS, если он по природе stateful + multi-component?

- **Контекст:** end-to-end тесты сложны (нужны cloud LLM mocks, симуляция long sessions, baseline benchmarks).
- **Кандидаты:**
  - Synthetic project generator (создаём «тестовый Django»).
  - Recorded inference replay.
  - Property-based testing для memory layers.
- **Решается в:** ADR-019 после первой имплементации памяти.

### Q8: Лицензия проекта?

- **Контекст:** owner ещё не сказал, open-source ли это или proprietary.
- **Решается в:** owner-decision до первого commit'а.

### Q9: Persistent ID-схема для фактов и эпизодов?

- **Контекст:** Если использовать UUID — humanly unreadable. Если slugs — collisions.
- **Кандидат:** hybrid (semantic prefix + nanoid suffix), e.g. `policy-money-decimal-7Hk2qP`.
- **Решается в:** ADR-020.

---

## Закрытые вопросы (для истории)

(Пусто — все решённые вопросы становятся ADR'ами, а не остаются здесь.)
