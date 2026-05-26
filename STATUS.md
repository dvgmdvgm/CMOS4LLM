# STATUS — где мы сейчас

> **Этот файл обновляется в конце каждой сессии работы над CMOS. 1 экран максимум.**
> Если STATUS не обновлён — следующая сессия начнётся со слепого понимания.

---

**Дата последнего обновления:** 2026-05-26
**Текущая фаза:** MVP implementation — Milestone 2 complete (~30% MVP)
**Кто работал:** owner + Opus 4.6

---

## Что сделано в последнюю сессию

1. **Тестирование LM-фаз с Ollama** — Gemma4 через Ollama работает, все 3 LM-фазы (Convention Mining, Rejected Approaches, Docs Ingestion) верифицированы. Граф: 5534 nodes, 45 edges.
2. **Unit tests для bootstrap pipeline** — 33 теста (PythonExtractor, DjangoExtractor, GraphStore) + 10 fixture файлов. `crates/bootstrap/tests/`.
3. **CI pipeline** — `.github/workflows/ci.yml`: clippy + test + build (Rust) + pnpm install + build (frontend). Все 25 clippy warnings исправлены.
4. **GitHub push** — репо `dvgmdvgm/CMOS4LLM`, branch `main`, все коммиты запушены.
5. **MVP Milestone 2: Memory layers L1–L4** — полная реализация в `crates/memory/` (5 модулей, ~1400 строк Rust, 30 тестов):
   - L1 Working Memory: lock-free in-memory buffer с priority-based eviction.
   - L2/L3 Event Store: SQLite WAL, temporal queries, session/entity/layer filtering.
   - L4 Project Memory: persistent fact store с tombstones и promotion tracking.
   - Promotion Engine: configurable L2→L3→L4 по access count + importance.

## Что НЕ сделано (ждёт следующей сессии)

- MVP Milestone 3: Two-LLM economy (Ollama runtime + background tasks + context assembly).
- Интеграция memory layers с CLI (пока нет команд для работы с L1–L4).
- Vector index (LanceDB) для semantic retrieval в L3/L4.

## Где мы в roadmap

- **Documentation phase:** 100% ✓
- **MVP Milestone 1 (Bootstrap):** 100% ✓
- **MVP Milestone 2 (Memory layers):** 100% ✓
- **MVP Milestone 3 (Two-LLM economy):** 0% — следующий шаг.
- **MVP implementation overall:** ~30%.

## Ключевые контекстные факты

- GitHub repo: `https://github.com/dvgmdvgm/CMOS4LLM.git`
- Owner использует Ollama с Gemma4 для Sub-LM задач.
- CI на Windows runner (GitHub Actions).
- Memory crate: L1 (RAM) + L2/L3 (SQLite WAL events.db) + L4 (SQLite facts.db).
- Promotion thresholds: L2→L3 (access≥3, importance≥0.6), L3→L4 (access≥5, importance≥0.8).

---

**Если только что открыл проект:** теперь читай [NEXT.md](./NEXT.md).
