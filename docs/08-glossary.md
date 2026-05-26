# CMOS Glossary

> Терминология проекта CMOS. Если термин используется в нескольких местах — он определён здесь.
> Если ты встретил незнакомый термин в любой документации — он должен быть здесь. Если нет — добавь.

---

## Core concepts

**CMOS** — Cognitive Memory Operating System. The entire system. External cognitive substrate sitting between human/IDE and LLM(s).

**Cognitive substrate** — the architectural layer that takes over what the transformer cannot do well: persistence, project identity, policies, observability.

**Two-LLM economy** — architectural pattern in CMOS: local Sub-LM does bulk cognitive work (extraction, summarization, dedup, linting, drift detection); cloud LLM only does critical reasoning (final code generation, architectural decisions). Inversion of typical practice where every chunk of text goes to the expensive model.

**Stateless LLM, stateful CMOS** — design principle. LLM is a pure function `f(context) → tokens`. CMOS is the source of truth for memory, identity, and history.

---

## Components

**Gateway** — single entry point. MCP server + HTTP/gRPC API + WebSocket. Auth, session routing, multi-tenancy. Implementation: Rust or Go.

**Context Hypervisor** — main orchestrator. Receives query → classifies task → plans retrieval → assembles prompt with budget enforcement → calls LLM → post-processes response. Most-trusted component.

**Retrieval Router** — selects and combines retrieval strategies. Not "one strategy" — produces a parallel plan.

**Constraint Solver** — enforces hard invariants. Constrained decoding for structured output, post-hoc validation + repair loop for free-form code.

**Sub-LM Runtime** — pool of local small models (3B–32B). Runs locally via llama.cpp / vLLM / MLX. Works batched and asynchronously in background.

**Policy & Invariant Engine** — storage and runtime for project rules. Three-tier (suggestions / soft / hard). Symbolic, not embedded.

**Observability & Telemetry** — first-class component. Cognitive traces, drift logs, token analytics, baseline comparison.

---

## Memory layers

**L1 — Working Memory** — what's currently being assembled into the prompt for the active inference call. RAM, ms latency, minutes TTL.

**L2 — Session Memory** — everything that happened in the current work session (typically a day). Event-sourced log + materialized views. RocksDB.

**L3 — Episodic / Task Memory** — memory about tasks (not sessions). Episode = a unit of work like "added cart abandonment email". Spans multiple sessions. Has lessons, rejected approaches, files touched, embedding for similar-task retrieval.

**L4 — Project Memory** — long-term project knowledge. Composite store: Symbol Graph + Semantic Vector Index + Knowledge Graph (domain ontology + temporal) + Policy Store + Project DNA. Never lost.

**L5 — Archival Memory** — everything that happened in project history. Cold storage, decay-with-versioning, ~1s retrieval. Object store + cold KG snapshot.

**Promotion** — moving memory items from a lower layer to a higher one (more permanent, more accessible).

**Demotion** — moving items down (typically L3 → L5 with TTL expiry).

**Tombstone** — soft-delete marker. CMOS never hard-deletes from L4/L5; deprecated items stay versioned.

---

## Project DNA

**Project DNA** — minimal complete description of project identity, always injected into LLM calls. 5K–20K tokens. Sections: Identity, Architectural pillars, Hard invariants, Style fingerprint, Forbidden patterns, Critical context.

**Constitution** — synonym for Project DNA in informal usage.

**Identity statement** — 2–4 lines describing what the product is, for whom, key value.

**Architectural pillars** — main design decisions with rationale (5–10 items).

**Style fingerprint** — naming, formatting, paradigm choices (10–20 items).

**Forbidden pattern** — explicitly prohibited approach with rationale (5–15 items).

---

## Policies

**Policy** — a rule. Structured object with id, type, scope, predicate, rationale, evidence_refs.

**Suggestion** — softest tier. Mentioned in prompt as a preference. No enforcement.

**Soft invariant** — middle tier. Mentioned in prompt + post-hoc check. Violation → warning, not block.

**Hard invariant** — strongest tier. Mentioned in prompt + constrained decoding (where applicable) + post-hoc check + repair loop or block on violation.

**Evidence** — link from a policy to its rationale source: decision IDs, incident IDs, PR refs, discussion logs. Used to answer "why does this rule exist?"

**Drift** — deviation from a policy. Detected by Sub-LM analyzing each generated diff.

**Drift Detection** — background process running on every diff to spot violations.

**Suggested rule** — a candidate policy proposed by drift trends ("this pattern was violated 12 times in 30 days → maybe codify as a rule").

---

## Retrieval & context assembly

**Symbolic pre-resolution** — answering queries deterministically without invoking LLM. E.g. "where is `foo` defined" → graph lookup, 0 LLM tokens.

**Semantic delta encoding** — between turn N and turn N+1, only what changed is sent to LLM. Already-seen items become short ID references.

**Compressed cognition block** — Sub-LM-extracted distillation of a complex reasoning episode. Reusable artifact, ~200–500 tokens replacing ~10K of raw dialog.

**Hierarchical summarization** — old turns get progressively compressed: recent → raw, old → summary, very old → digest.

**Lazy loading via references** — instead of injecting file content, inject `<REF path: outline>`; LLM decides if it needs full content.

**Cognitive replay skip** — if task is identical (high semantic similarity) to a recently-solved one, return cached response without re-invoking LLM.

**Knapsack assembly** — budget-aware optimization: select context items maximizing relevance under token budget.

**Attention-aware prompt rendering** — critical items at start/end (where attention is strongest); less critical in middle. Imperatives over prose.

**Budget** — token cap on assembled context. MVP default: 32K.

---

## Episodes & sessions

**Session** — a contiguous work period (typically a day or task). Has a start, events, an end. L2-resident.

**Episode** — a unit of work (a "task"), possibly spanning multiple sessions. Has problem statement, approaches taken/rejected, lessons, outcome. L3-resident, may promote to L4 as case study.

**Lesson** — extracted insight from an episode. Reusable for future similar tasks.

**Case study** — high-value episode promoted to L4 for permanent reference.

---

## GUI

**Cognitive Console** — the GUI as a whole. Tauri hybrid shell + web core + IDE plugins.

**Live Inference Inspector** — flagship GUI screen. Real-time view of prompt assembly, streaming response, post-gen validation.

**Cognitive Trace overlay** — always-on small widget in lower-right corner showing current activity.

**Time Travel Debugging** — ability to open a past inference call and see its full assembled context, replay it, or run counterfactuals.

**Counterfactual mode** — re-run past N inferences with alternative policies; compare outcomes. A/B testing for policies.

**Memory Heatmap** — visualization of memory items colored by access frequency. Hot items red, cold items grey.

**Drift Monitor** — GUI screen showing policy violations over time, trends, and suggested rules.

**Project DNA Editor** — versioned editor for the constitution. Diff viewer, evidence links, suggested rules from drift.

---

## Process & lifecycle

**Bootstrap** — initial onboarding pipeline for an existing project. Builds L4 from static analysis, git history, conventions, documentation, and interactive elicitation.

**Convention mining** — Sub-LM extracts de-facto rules from existing code (naming, paradigms, style).

**Elicitation** — interactive Q&A with owner to fill in policies that can't be inferred from code.

**Consolidation** — periodic background job (Sub-LM): dedup facts, resolve contradictions, update DNA.

**Conflict resolution** — protocol for new fact contradicting existing one. Never overwrite; ask or auto-resolve via recency / version chain.

**Wake-up resilience** — non-functional requirement: any new chat window must restore project context within 5 minutes of reading top-level docs.

---

## Documentation

**ADR** — Architecture Decision Record. One file per decision in `docs/02-decisions/`. Includes context, decision, rationale, consequences, alternatives.

**Conversation log** — dated entry in `docs/09-conversation-log/` recording a session's key decisions and reasoning.

**STATUS.md** — living document, where we are now. Updated each session.

**NEXT.md** — living document, next 3–5 steps. Updated each session.

**ROADBLOCKS.md** — open questions blocking progress. Each closed question becomes an ADR.

---

## External terms (used loosely in docs, defined here for precision)

**MCP** — Model Context Protocol (Anthropic). Standard for connecting tools/resources to LLMs. CMOS is implemented as an MCP server (among other interfaces).

**KV-cache** — internal key/value tensors of the transformer attention mechanism. Sometimes reusable within a single inference instance (prompt caching), but not transferable across sessions.

**Prompt caching** — provider-side feature reusing KV computations for identical prompt prefixes within TTL (Anthropic 5min, OpenAI similar, Gemini context caching).

**RAG** — Retrieval-Augmented Generation. Treated in CMOS as one (demoted) retrieval primitive among many, not the architecture.

**Embedding** — vector representation of text used for similarity search. Lossy compression. Good for fuzzy similarity, weak for exact/relational/temporal/constraint queries.
