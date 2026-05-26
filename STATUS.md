# STATUS — где мы сейчас

> **Этот файл обновляется в конце каждой сессии работы над CMOS. 1 экран максимум.**
> Если STATUS не обновлён — следующая сессия начнётся со слепого понимания.

---

**Дата последнего обновления:** 2026-05-26
**Текущая фаза:** MVP implementation — Milestone 1 complete (~15% MVP)
**Кто работал:** owner + Opus 4.6

---

## Что сделано в последнюю сессию

1. **MVP Milestone 1: Bootstrap pipeline** — полная реализация 8-фазного pipeline для онбординга проектов в L4 knowledge graph. Crate `cmos-bootstrap` (27 файлов, ~3800 строк Rust).
2. **Верифицировано на реальном Django-проекте** (224 Python файла, `D:\art_network_antigravity`): 4956 nodes, 45 edges, 43 git hotspots. Статические фазы завершаются за ~25 секунд.
3. **CLI команды** — `cmos bootstrap`, `cmos graph stats`, `cmos graph query` работают.
4. **Design spec** — `docs/superpowers/specs/2026-05-26-bootstrap-pipeline-design.md` закоммичен.

## Что НЕ сделано (ждёт следующей сессии)

- LM-фазы (4, 6, 7) не тестировались с Ollama (Ollama не был запущен в момент тестирования).
- Unit tests / integration tests (pipeline работает, но формальных тестов нет).
- CI pipeline (GitHub Actions).
- MVP Milestone 2: Memory layers L1–L4.

## Где мы в roadmap

- **Documentation phase:** 100% ✓
- **MVP Milestone 1 (Bootstrap):** 100% ✓ — pipeline работает на реальном проекте.
- **MVP Milestone 2 (Memory layers):** 0% — следующий шаг.
- **MVP implementation overall:** ~15%.

## Ключевые контекстные факты

- Owner использует Ollama с Gemma2 для Sub-LM задач.
- Django-проект для тестирования: `D:\art_network_antigravity` (Scenica — marketplace для артистов).
- Bootstrap создаёт `.cmos/config.toml` и `.cmos/graph.db` в корне target project.
- Классификатор Django: models, views, admins, serializers, forms, middleware, signals, consumers, management commands.
- InferenceBackend trait: OllamaBackend + ApiBackend + MockBackend.

---

**Если только что открыл проект:** теперь читай [NEXT.md](./NEXT.md).
