# 2026-05-26 — Architecture document and documentation skeletons

> Сессия по закрытию пунктов 1 и 2 из NEXT.md: мастер-документ архитектуры + скелеты всех оставшихся разделов документации.

---

## Контекст сессии

- **Owner** дал команду «переходим к реализации» — wake-up + сразу за работу.
- Задача: закрыть два пункта из NEXT.md, которые завершают documentation phase на ~95%.

---

## Что сделано

### 1. `docs/01-architecture.md` — мастер-документ архитектуры

Полноценный технический документ на английском, содержащий:

- **System Overview** — core thesis (stateless LLM + CMOS as source of truth), Two-LLM economy, multi-project isolation.
- **Component Map** — Mermaid-диаграмма всех компонентов с потоками данных.
- **Memory Hierarchy** — таблица L1–L5 с детальными свойствами, promotion/demotion, conflict resolution, immutability.
- **Inference Pipeline** — sequence diagram (Mermaid) от user request до response, с latency budget table (<200ms p95 pre-LLM).
- **Token Reduction Techniques** — все 12 техник с reduction estimate, complexity, component owner, scope reference. Пример композиции на реальной задаче.
- **Project DNA & Policy Engine** — DNA structure (6 sections), three-tier policy model, drift detection.
- **Observability & Time Travel** — InferenceRecord schema, time travel API, counterfactual mode.
- **Integration Architecture** — protocol hierarchy (MCP/HTTP/gRPC), GUI architecture diagram (Tauri + web core + IDE plugins).
- **Bootstrap Pipeline** — 8-step diagram с указанием LLM/no-LLM per step.
- **Cross-Reference Index** — ADR→component mapping table, scope→architecture mapping table.
- **Non-Functional Requirements** — latency, token reduction targets, bootstrap time, binary size.
- **Security & Privacy** — no raw chat to cloud, privacy mode, local-first, auth.

### 2. Скелеты документации (27 файлов)

**`docs/04-components/`** (8 файлов):
- `gateway.md`, `cognitive-hypervisor.md`, `retrieval-router.md`, `memory-layers.md`, `policy-engine.md`, `sub-lm-runtime.md`, `observability.md`, `bootstrap-pipeline.md`.

Каждый файл содержит: Responsibility, ключевые секции с TODO, Dependencies, Scope (ссылки на MVP/V1/V2), Open Questions.

**`docs/05-gui/`** (12 файлов):
- `dashboard.md`, `live-inspector.md`, `memory-browser.md`, `knowledge-graph.md`, `dna-editor.md`, `drift-monitor.md`, `token-analytics.md`, `episodes-browser.md`, `policy-manager.md`, `cognitive-trace-overlay.md`, `design-system.md`, `theming.md`.

Каждый файл содержит: Purpose, Key Features (TODO), Data Sources, Scope (MVP vs V1 vs V2), Open Questions.

**`docs/06-bootstrap/`** (1 файл):
- `django-marketplace.md` — детальный 8-step pipeline с TODO per step, acceptance criteria, performance target (<8h for 400K LoC).

**`docs/07-research/`** (6 файлов):
- `kv-cache-persistence.md`, `lora-as-memory.md`, `neurosymbolic.md`, `latent-persistence.md`, `external-attention.md`, `distributed-cognition.md`.

Каждый файл содержит: Hypothesis, Key Challenges (TODO), Related Work (TODO), CMOS Integration Points, Dependencies, Status.

---

## Ключевые решения

Архитектурных решений, требующих нового ADR, в этой сессии не принималось. Вся работа — документирование уже принятых решений из предыдущей сессии.

---

## Что дальше

Следующий шаг — **ADR-011..016 (выбор стека)**. Это последний блок documentation phase перед началом имплементации. После него — git init + структура исходников + первый код (MVP M1: bootstrap pipeline).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Результат:** documentation phase ~95% complete. Остаётся только выбор стека (ADR-011..016).
