# ROADMAP — MVP / V2 / V3 / Future

> Этот документ — **граница между фазами**. Каждая фича попадает в одну из четырёх корзин. Без размытости.
> Если что-то «надо бы сделать» и непонятно, в какую фазу — оно идёт в Future до тех пор, пока не будет осознанно повышено.

---

## Принципы планирования

1. **MVP отвечает на вопрос: «работает ли вообще?»** — не «удобно ли», не «красиво ли». MVP — proof of substrate.
2. **V1 (= release candidate)** — то, что мы реально хотим использовать на Django marketplace в production-режиме.
3. **V2** — фичи, без которых жить можно, но с которыми CMOS становится breakthrough-инструментом.
4. **V3** — frontier, требует исследовательской работы.
5. **Future** — всё остальное. Когда дойдёт — тогда дойдёт.

---

## MVP — proof of substrate

**Цель MVP:** доказать, что Django marketplace может работать через CMOS-слой и токены реально снижаются. Не больше.

### MVP включает

- **Bootstrap pipeline для существующего проекта** (Django-aware):
  - AST-парсинг всего Python кода.
  - Извлечение моделей, views, urls, signals, settings, migrations.
  - Построение L4 Symbol Graph.
  - Извлечение де-факто конвенций (Sub-LM, batch-режим).
  - Mining git history (commits + diffs).
  - Interactive elicitation (~20 вопросов) → начальный Project DNA.
- **Memory hierarchy L1–L4** (L5 — minimal, log-only):
  - L1 working memory (in-process).
  - L2 session memory (RocksDB + event log).
  - L3 episodic memory (с похожестным retrieval).
  - L4 project memory: symbol graph + vector index + policy store + DNA.
- **Two-LLM economy:**
  - Local Sub-LM runtime (llama.cpp + Qwen 2.5 Coder 14B как baseline).
  - Cloud LLM intermediation через MCP server.
- **Context Hypervisor**:
  - Простая retrieval orchestration (graph + vector + policy).
  - Token budget enforcement (knapsack).
  - Attention-aware prompt assembly (policies в начало/конец).
- **Token reduction techniques (subset):**
  - Sub-LM pre-filtering.
  - Symbolic pre-resolution.
  - Hierarchical summarization.
  - Prompt caching awareness (Anthropic).
  - Lazy loading через references.
- **Policy & Invariant Engine:**
  - Soft policies (injection only).
  - Hard invariants (post-hoc validation + repair loop, без constrained decoding).
  - DNA store (versioned).
- **Multi-project из коробки:**
  - Project switcher.
  - Изолированные namespaces для памяти/политик/DNA.
- **GUI MVP — высокая плотность, гибрид Tauri:**
  - Dashboard.
  - Live Inference Inspector (флагман).
  - Memory Browser.
  - Token Analytics.
  - Cognitive Trace overlay.
  - Time Travel Debugging (как обещано — в V1).
- **MCP Server** для подключения к Claude Code / Cursor / любого MCP-клиента.

### MVP НЕ включает (явно)

- DNA editor GUI (CLI редактирование DNA достаточно для MVP).
- Drift Monitor GUI (логи drift есть, но без отдельного экрана).
- Knowledge Graph Viewer (граф пишется, но не визуализируется).
- Counterfactual mode (V1, потому что обещали).
- Episodes Browser GUI (CLI достаточно).
- Constrained decoding (только post-hoc validation).
- L5 Archival (только append-only лог; без cold KG, без decay).
- IDE plugins (VS Code / JetBrains — V2).
- Cross-project memory transfer.

---

## V1 — release candidate (то, что используем сами)

Достраиваем то, что обещали в дизайне, но было unrealistic для MVP.

### V1 добавляет

- **Drift Monitor GUI** + автоматический drift log → suggested rules.
- **DNA Editor GUI** с versioning, diff viewer, evidence links.
- **Knowledge Graph Viewer** (Cytoscape, режимы Domain / Code / Decisions / Combined).
- **Episodes Browser GUI** + lessons mining.
- **Policy Manager GUI** + bulk operations.
- **Counterfactual mode** (Sub-LM в background перепрогоняет последние N inferences с альтернативным набором политик).
- **VS Code extension** (тот же React webview).
- **L5 Archival** полноценный (cold storage + decay + versioned KG).
- **Compressed cognition blocks** (reusable cognition).
- **Differential retrieval**.
- **Constraint hoisting** (basic — структурный output для tool calls).
- **Memory Heatmap** (frequency-based visualization).
- **Bootstrap pipelines** для второго типа проектов (Node/TypeScript) — подтверждаем универсальность.

### V1 НЕ включает

- Persistent KV-cache хитрости (требует кооперации с провайдером).
- LoRA-as-memory.
- JetBrains plugin.
- Manager view для нетехнических.
- Multi-user collaboration.

---

## V2 — breakthrough features

### V2 добавляет

- **Constrained decoding** через локальный inference proxy (грамматики, JSON schema, custom DSL).
- **JetBrains plugin**.
- **Manager / shared view** (упрощённая поверхность для нетехнических ролей).
- **Multi-user collaboration** (shared memory, comments на episodes/decisions, attribution).
- **Cross-project memory** (transfer learning между проектами одного владельца).
- **Auto-policy promotion** (drift trends → автоматическое предложение новой политики).
- **A/B policy testing** в production (group A с правилом, group B без, метрики).
- **Cost dashboard для команд** (per-user / per-project budgets).
- **Self-optimization layer:** cognitive profiler + token waste detector + adaptive memory hierarchy.

---

## V3 — frontier

Требует исследовательской работы или партнёрств.

- **Persistent KV-cache хитрости:** sticky routing к inference node, расширение TTL до часов.
- **LoRA-as-memory:** дистилляция «проектной модели» после года работы.
- **Symbolic-neural hybrid reasoning:** SMT/Z3 интеграция для логических ограничений.
- **Latent memory persistence research** (Mamba-Hybrid эксперименты).
- **External attention systems** (research collaborations).
- **Distributed cognition** (multi-agent coordination через CMOS как shared substrate).

---

## Future — отложено

- Mobile UI.
- Standalone SaaS hosting.
- Marketplace для project DNA templates (Django, FastAPI, Next.js…).
- Plugin SDK для third-party retrieval strategies.
- Federated memory (несколько CMOS-инстансов синхронизируют шарящуюся часть).

---

## Анти-плавающий scope

Если новая идея не попадает в текущую активную фазу — она идёт в Future. Повышение фаз — осознанное решение через ADR. Без «давайте быстро добавим X» в MVP.
