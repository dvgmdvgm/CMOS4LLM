# Scope: MVP — Proof of Substrate

> **Goal of MVP:** demonstrate that a real Django marketplace can be driven through the CMOS layer with measurably lower cloud token cost and persistent project memory across sessions. Not "nice to use" — "is the substrate real?"
>
> Anything not listed here is **not in MVP**. See [v1.md](./v1.md) and beyond. Items with explicit acceptance criteria below — if a criterion is not met, the feature is not done.

---

## In scope (with acceptance criteria)

### M1. Bootstrap pipeline (Django-aware)

**Goal:** turn a 100K–1M LoC Django repository into a functional L4 in hours, not days.

- [ ] Static AST sweep over all `.py` files; extract: classes, functions, imports, decorators.
- [ ] Django-specific extractors: `models.py` (fields, FK, indexes, Meta), `urls.py` (route table), `views.py` (CBV vs FBV detection, mixins), `signals.py`, `apps.py`, `settings.py` (env-aware), all `migrations/`.
- [ ] L4 Symbol Graph populated; queryable: "who calls X", "who depends on Y", "URLs serving model Z".
- [ ] Domain ontology bootstrap from `models.py` — every model becomes a domain entity with FK relations as edges.
- [ ] Convention mining (Sub-LM, batched): naming patterns, function size distribution, paradigm dominance (CBV/FBV %), test layout, import style.
- [ ] Git-history mining: commit log + diff stats per file; revert-pattern detection; merge-vs-rebase preference.
- [ ] Documentation ingestion: `README*`, `docs/`, `CHANGELOG*`, `ADR*`, `*.md` files.
- [ ] Interactive elicitation script: 20–50 owner questions to seed initial DNA. CLI is acceptable (no GUI in MVP).

**Acceptance:** owner runs `cmos bootstrap --project marketplace --root <repo>` on the actual Django marketplace repo. After completion, `cmos memory show --project marketplace --layer L4 --filter symbols` returns a non-empty graph; querying any well-known domain entity (User, Order, Product) returns its dependencies; initial DNA exists and is human-reviewable.

### M2. Memory layers L1–L4 (L5 = log-only)

- [ ] **L1** — in-process, RAM, lock-free; assembled prompt + per-turn scratch.
- [ ] **L2** — RocksDB (or SQLite WAL fallback) event log; immutable events; materialized views derivable.
- [ ] **L3** — episodes with vector index for similar-task retrieval.
- [ ] **L4** — composite store: graph DB (per ADR-012 — TBD), vector index (per ADR-013 — TBD), KV store. All in-process.
- [ ] **L5** — append-only log only. No cold KG, no decay, no retrieval optimization. Just safe storage.
- [ ] Promotion logic L2→L3, L3→L4 implemented (recency + access count + Sub-LM importance score).
- [ ] Tombstones supported in L4; no hard-delete (per ADR-009).

**Acceptance:** end-to-end test — owner closes CMOS, reboots machine, opens 3 days later; `cmos memory show --project marketplace` returns identical L4 contents; new session sees L4 facts injected into context on relevant queries.

### M3. Two-LLM economy (per ADR-003)

- [ ] Sub-LM Runtime: pool of one local model (14B class) via `llama.cpp` or `vLLM`. Hot-swappable model file.
- [ ] Cloud LLM intermediation: Anthropic API (Opus / Haiku) at minimum.
- [ ] Sub-LM tasks running in MVP: classification, extraction, summarization, dedup, lint check.
- [ ] Background queue with priority: live path > consolidation > drift scan > counterfactual (in MVP only the first two operate).
- [ ] Fallback profile: if no local GPU detected, dispatch Sub-LM tasks to cheap cloud model (Haiku 4.5).

**Acceptance:** Sub-LM extracts ≥80% of facts on a hand-labeled sample of 100 commits from the Django marketplace, measured against the owner's expected facts.

### M4. Context Hypervisor

- [ ] Task classification (Sub-LM): `code_modification | code_question | architectural | debug | docs | other`.
- [ ] Retrieval planning: rule-based plan per task class; calls `Retrieval Router`.
- [ ] Token budget enforcement via knapsack (MVP default budget: 32K).
- [ ] Attention-aware prompt rendering: hard policies in start + tail; deltas in body; less-critical in middle.
- [ ] Latency budget on critical path: assembly < 200ms p95 (Sub-LM excluded; Sub-LM calls run async).

**Acceptance:** synthetic benchmark of 50 typical Django tasks shows: every assembled prompt ≤ 32K tokens; p95 assembly latency < 200ms on owner's hardware; every prompt contains all hard policies in scope at start + tail position.

### M5. Token reduction techniques (MVP subset)

- [ ] **Sub-LM pre-filtering**: every cloud call's context first passes through Sub-LM compression/filtering.
- [ ] **Symbolic pre-resolution**: deterministic queries (graph lookups) bypass LLM entirely.
- [ ] **Hierarchical summarization**: in-session turn compression (recent → raw, old → summary, very old → digest).
- [ ] **Prompt caching awareness**: stable prefix structured to maximize Anthropic 5min cache hits.
- [ ] **Lazy loading via references**: file content replaced by `<REF path: outline>`; LLM tool-calls to expand.

**Not in MVP:** semantic delta encoding, compressed cognition blocks, differential retrieval, constraint hoisting, cognitive replay skip, persistent KV. (See [v1.md](./v1.md), [v2.md](./v2.md).)

**Acceptance:** measured against simulated baseline (same query without CMOS), MVP achieves ≥3× cloud-token reduction on the owner's typical Django tasks (lower bound; target for V1 is 5–25×).

### M6. Policy & Invariant Engine

- [ ] Policy schema: `id, type {suggestion|soft|hard}, scope, predicate, rationale, evidence_refs, created_at, version`.
- [ ] Soft policies: prompt injection only; no enforcement.
- [ ] Hard invariants: prompt injection + post-hoc validation via Sub-LM + repair loop (one retry).
- [ ] DNA store: versioned, append-only, human-editable via CLI.
- [ ] Constrained decoding: NOT in MVP (post-hoc validation only).

**Acceptance:** seeded with 10 hard invariants from elicitation, the system blocks an obvious violation in a generated diff (e.g., `float` for money) within the repair loop.

### M7. Multi-project (per ADR-004)

- [ ] `project_id` non-null in every storage row, KG label, partition key.
- [ ] CLI requires `--project <slug>`; no implicit current project.
- [ ] Gateway sessions are project-scoped.
- [ ] Project switcher in GUI header.

**Acceptance:** create project A and project B; ingest different DNA into each; verify zero cross-project leakage in retrieval and policy enforcement.

### M8. GUI MVP — high density (per ADR-006), Tauri hybrid (per ADR-007)

Screens that ship in MVP:

- [ ] **Dashboard** — token economy, memory health, drift counts, recent episodes, active policies.
- [ ] **Live Inference Inspector** — streaming view: assembled context (with relevance scores per item), excluded items + reason, cache hit ratio, post-gen validation. Replay button.
- [ ] **Memory Browser** — layer tree, item list with filter/search, detail panel with evidence, usage stats, version history.
- [ ] **Token Analytics** — headline numbers, savings breakdown by technique, daily timeseries vs simulated baseline.
- [ ] **Cognitive Trace overlay** — always-on, lower-right, recent activity feed.

**Acceptance:** all five screens functional, populated with real data from the marketplace project, refreshing in real time over WebSocket.

### M9. Time Travel Debugging (per ADR-005)

- [ ] Every inference call recorded as immutable `InferenceRecord` with full assembled prompt, response, retrieved items + scores, active policy IDs, DNA version, timestamp, project_id.
- [ ] Live Inference Inspector supports navigating past records (filter by project, search by query content).
- [ ] Replay button: re-runs assembly with same inputs, shows diff if any (should be deterministic at MVP scope).

**Acceptance:** open an inference from 2 days ago; full original context reconstructs; replay produces identical assembled prompt.

### M10. MCP server (per ADR-010)

- [ ] CMOS daemon exposes MCP server endpoint on configurable port.
- [ ] Resources: `cmos://project/{id}/dna`, `cmos://project/{id}/policies`, `cmos://project/{id}/episodes/*`.
- [ ] Tools: `cmos.assemble_context`, `cmos.record_decision`, `cmos.find_similar_episodes`, `cmos.validate_against_policies`, `cmos.time_travel`.
- [ ] Tested with at least: Claude Code (claude-code CLI), Cursor, Claude Desktop.

**Acceptance:** owner uses Claude Code → CMOS via MCP to perform a typical task on the marketplace; CMOS-assembled context is observably present in the conversation.

### M11. Documentation discipline

- [ ] All architectural decisions made during MVP captured as ADRs.
- [ ] Bootstrap procedure documented in `docs/06-bootstrap/django-marketplace.md`.
- [ ] STATUS.md, NEXT.md, conversation-log discipline in effect — see [CLAUDE.md](../../CLAUDE.md).

---

## Out of MVP (explicitly)

These are real features, but they are **not in MVP** and any push to include them blocks the release:

- DNA Editor GUI (CLI editing only in MVP)
- Drift Monitor GUI (logs exist, no dedicated screen)
- Knowledge Graph Viewer (graph exists, not visualized)
- Counterfactual mode (V1)
- Episodes Browser GUI (CLI only)
- Policy Manager GUI (CLI only)
- Constrained decoding
- L5 Archival (full) — only log-only stub in MVP
- VS Code / JetBrains plugins (V1 / V2)
- Cross-project memory transfer
- Compressed cognition blocks
- Memory Heatmap

---

## Definition of Done for MVP

All of the following must be true:

1. Owner can boot CMOS daemon, run `cmos bootstrap` on the Django marketplace, and complete bootstrap in < 8 hours of compute.
2. Owner can use Claude Code → CMOS (via MCP) for daily marketplace work without bypassing CMOS.
3. Memory survives session boundaries (cold-restart + 3-day pause + cold-restart yields identical L4).
4. Token Analytics shows ≥3× reduction vs simulated baseline on a representative task set of 30 inferences.
5. All 5 MVP GUI screens render with real data, no crashes for 8h continuous use.
6. Time Travel: any past inference can be opened, replayed, and shows identical assembled context.
7. Multi-project: zero leakage in test (project A + project B isolation).
8. Documentation: STATUS / NEXT / ROADMAP / ROADBLOCKS / ADRs all current; conversation-log of every working session.

---

## Order of work (suggested)

1. **Skeleton + CI** (repo, build, lint).
2. **Bootstrap pipeline** for Django (M1) — first valuable artifact.
3. **L4 + L2 storage** (M2) — needed by everything.
4. **Sub-LM Runtime** (M3) — feeds bootstrap and ongoing.
5. **Hypervisor + Retrieval Router** (M4) — the orchestrator.
6. **Token reduction subset** (M5) — measurable wins.
7. **Policy Engine + DNA store** (M6).
8. **MCP server** (M10) — the integration surface.
9. **GUI MVP screens** (M8) + **Time Travel substrate** (M9, schema must be in place from step 3).
10. **Multi-project hardening** (M7) — should be implicit from step 1 but verified end-to-end.

Steps 7–10 can substantially overlap once 1–6 are stable.
