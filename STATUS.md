# STATUS — где мы сейчас

> **Этот файл обновляется в конце каждой сессии работы над CMOS. 1 экран максимум.**
> Если STATUS не обновлён — следующая сессия начнётся со слепого понимания.

---

**Дата последнего обновления:** 2026-05-26
**Текущая фаза:** Documentation phase — ~95% (architecture + все скелеты готовы; осталось ADR-011..016)
**Кто работал:** owner + Opus 4.6

---

## Что сделано в последнюю сессию

1. **Написан `docs/01-architecture.md`** — мастер-документ архитектуры на английском:
   - System overview, component map (Mermaid), memory hierarchy, inference pipeline (sequence diagram), 12 token reduction techniques, Project DNA & Policy Engine, observability & time travel, integration architecture, bootstrap pipeline, cross-reference index (ADR→component, scope→architecture), NFRs, security.
2. **Созданы скелеты `docs/04-components/`** (8 файлов):
   - `gateway.md`, `cognitive-hypervisor.md`, `retrieval-router.md`, `memory-layers.md`, `policy-engine.md`, `sub-lm-runtime.md`, `observability.md`, `bootstrap-pipeline.md`.
3. **Созданы скелеты `docs/05-gui/`** (12 файлов):
   - `dashboard.md`, `live-inspector.md`, `memory-browser.md`, `knowledge-graph.md`, `dna-editor.md`, `drift-monitor.md`, `token-analytics.md`, `episodes-browser.md`, `policy-manager.md`, `cognitive-trace-overlay.md`, `design-system.md`, `theming.md`.
4. **Создан `docs/06-bootstrap/django-marketplace.md`** — детальный pipeline для 400K LoC Django.
5. **Созданы скелеты `docs/07-research/`** (6 файлов):
   - `kv-cache-persistence.md`, `lora-as-memory.md`, `neurosymbolic.md`, `latent-persistence.md`, `external-attention.md`, `distributed-cognition.md`.

## Что НЕ сделано (ждёт следующей сессии)

- Технологические ADR-011..ADR-016 (выбор стека: язык, graph DB, vector index, Sub-LM runtime, GUI shell, storage backend).
- Git init + стартовая структура исходников.
- MVP Milestone 1: Bootstrap pipeline (первый исполнимый код).

## Где мы в roadmap

- **Documentation phase:** ~95% (charter ✓, ADR-001..010 ✓, scope ✓, glossary ✓, architecture ✓, component skeletons ✓, GUI skeletons ✓, bootstrap spec ✓, research skeletons ✓; осталось: ADR-011..016 по стеку).
- **MVP implementation:** не начато.

## Ключевые контекстные факты

- Owner — разработчик Django-marketplace ~400K LoC. CMOS будет применяться к этому проекту первым.
- Wake-up resilience — главное правило: документация спроектирована так, что любое окно чата восстанавливает контекст за 5 минут.
- Авто-протоколы: на первое сообщение Claude обязан прочитать STATUS+NEXT+последний conversation-log и доложить. На триггер-фразы — сразу за работу. На завершение задачи / `сохраняемся` — обязательный sleep-ритуал.
- Язык документации: charter и conversation-log на русском; spec и ADR — на английском.

---

**Если только что открыл проект:** теперь читай [NEXT.md](./NEXT.md).
