# 2026-05-26 — Scope finalization & wake/sleep automation

## Контекст

Это вторая сессия работы над CMOS в один день. Первая (см. `2026-05-26-initial-design.md`) закрыла этапы 1–6 архитектурного исследования и зафиксировала первые 10 ADR + структуру документации. На вход этой сессии:

- Charter (`docs/00-charter.md`) ✓
- Glossary (`docs/08-glossary.md`) ✓
- ADR-001..010 ✓
- Top-level anti-forgetting (README, STATUS, NEXT, ROADMAP, ROADBLOCKS, CLAUDE) ✓
- Папки `docs/03-scope/`, `docs/04-components/`, `docs/05-gui/`, `docs/06-bootstrap/`, `docs/07-research/` пустые.

Owner за время сессии задал важный вопрос про процесс: «как мне начинать новую сессию каждый раз, чтобы Claude автоматически понимал, на каком этапе мы». Это сместило фокус: помимо закрытия scope, нужно зафиксировать **исполняемый протокол wake/sleep**, иначе wake-up resilience остаётся декларацией, а не реальностью.

## Что сделано

### Scope-файлы (`docs/03-scope/`)

- `mvp.md` — 11 milestones M1–M11 с явными acceptance criteria. Главная цель MVP: ≥3× cloud-token reduction на воспроизводимом benchmark на Django marketplace.
- `v1.md` — 13 фич V1.A–V1.M. Главная цель V1: ≥10× weighted aggregate token reduction; owner использует CMOS как daily driver ≥30 рабочих дней без fallback к direct LLM.
- `v2.md` — 9 фич V2.A–V2.I (constrained decoding через local proxy, JetBrains, manager view, multi-user, cross-project transfer, auto-policy promotion, A/B testing, cost dashboard, self-optimization layer).
- `v3.md` — 6 research-coupled треков V3.A–V3.F (KV-cache persistence, LoRA-as-memory, neurosymbolic, latent persistence, external attention, distributed cognition). Acceptance включает «publishable result», не только «фича работает».
- `future.md` — 10 deferred items (mobile, SaaS, marketplace, plugin SDK, federation, voice, career-scale memory, adversarial robustness, regional pinning, PM tool integration). Промоушн в фазу — через ADR.
- `out-of-scope.md` — 12 категорических non-goals. **Не deferred**, а **fundamental**: «CMOS не модифицирует модели», «не пишет код за пользователя», «не заменяет code review», «не auto-pilot refactor», «не competing chat UI», «не general-purpose chat memory», «не database», «не toy», «не silently accepts hallucinations», «не bypasses owner authority», «not free of cost».

### Wake/sleep автоматизация

- **`CLAUDE.md` переписан как исполняемый протокол** (не «шпаргалка»). Добавлены два HARD GATE-блока:
  - **WAKE-UP RITUAL** — на первое содержательное сообщение сессии Claude **обязан** прочитать STATUS+NEXT+последний conversation-log+ROADBLOCKS и доложить одним абзацем. Не опционально.
  - **SLEEP RITUAL** — на завершение задачи / триггер-фразу `сохраняемся` / `закрываем сессию` Claude **обязан** обновить STATUS, NEXT, создать новый conversation-log, при необходимости — ADR / ROADBLOCKS.
- **Триггер-фразы** для перехода wake-up → сразу к работе: `переходим к реализации`, `продолжаем`, `продолжай`, `дальше`, `next`, `go`. После них Claude делает короткий wake-up отчёт и берёт первый незакрытый пункт из NEXT.md без переспроса.
- **Slash-команды** созданы в `.claude/commands/`:
  - `wake.md` — явный запуск wake-up (например, после длинной паузы внутри сессии).
  - `sleep.md` — явный запуск sleep (по строгому шестишаговому протоколу).

## Ключевые решения

### Зачем «исполняемый протокол» вместо «правил памяти»

В первой версии `CLAUDE.md` было «помни про wake-up resilience», но это формулировка-намерение, а не команда. Owner справедливо заметил: «можно сделать так, чтобы я каждую новую сессию начинал просто со слов *переходим к реализации* и Claude автоматически понимал, на каком этапе мы». Это требует hard gate с явным списком действий и триггеров — иначе модель в каждой новой сессии будет принимать решение «читать или не читать» по-разному.

**Решение:** CLAUDE.md теперь содержит два явных gate-блока (`⛔ HARD GATE — WAKE-UP RITUAL` и `⛔ HARD GATE — SLEEP RITUAL`) с конкретными шагами, триггерами и проверяемыми артефактами. Это превращает wake-up resilience из обещания в работающий механизм.

### Зачем slash-команды, если есть auto-протоколы

Auto-протоколы покрывают 95% случаев (новая сессия, завершение задачи). Но иногда нужно **принудительно** прогнать ритуал — например, owner заходит в существующее окно после длинной паузы, и хочет «перепроверить», а не продолжать. Или owner хочет явно сохраниться перед перерывом, не закрывая окно. Slash-команды — это «кнопка», auto-протоколы — это «реакция на сигналы».

### Acceptance criteria в каждой scope-фиче

Без них scope drift становится неотличим от прогресса. Каждая фича V1.A..V1.M, V2.A..V2.I, V3.A..V3.F имеет measurable acceptance — формулируется до начала имплементации. Если фича не проходит — она не считается сделанной, даже если «вроде работает».

### Out-of-scope vs Future

Out-of-scope — это **категорические** non-goals, не deferred. Если запрос мапится в один из 12 пунктов out-of-scope — ответ «нет», независимо от фазы. Future — наоборот, открытая дверь: путь открыт, но требует ADR для промоушена. Это разделение защищает от «давайте быстро добавим X», когда X фундаментально не вписывается в идентичность CMOS.

## Открытые вопросы

Никаких новых блокеров не появилось. Активные вопросы остаются те же (см. `ROADBLOCKS.md`):
- Q1 — Sub-LM модель.
- Q2 — точный token budget (решится после первых benchmark'ов).
- Q3 — privacy mode для cloud calls (PII / payment в production codebase).
- Q4 — DNA ownership (single user vs team) — решится в V2.
- Q5 — резолвинг конфликтов между версиями DNA.
- Q6 — bootstrap без git history.
- Q7 — стратегия тестирования stateful multi-component системы.
- Q8 — лицензия проекта (open-source vs proprietary).
- Q9 — persistent ID-схема для фактов и эпизодов.

## Следующий шаг

Согласно `NEXT.md`:
1. **`docs/01-architecture.md`** — мастер-документ архитектуры (английский, со ссылками на ADR-001..010 и scope-файлы).
2. Скелеты `docs/04-components/`, `docs/05-gui/`, `docs/06-bootstrap/`, `docs/07-research/`.

Это закроет documentation phase. Дальше — технические ADR-011..016 (выбор стека) и git init.

## Артефакты сессии

- Создано: `docs/03-scope/mvp.md`, `v1.md`, `v2.md`, `v3.md`, `future.md`, `out-of-scope.md`.
- Создано: `.claude/commands/wake.md`, `.claude/commands/sleep.md`.
- Обновлено: `CLAUDE.md` (полный rewrite с auto-протоколами).
- Обновлено: `STATUS.md`, `NEXT.md` (отражают текущее состояние).
- Создано: этот файл (`2026-05-26-scope-and-automation.md`).
