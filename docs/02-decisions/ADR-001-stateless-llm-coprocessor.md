# ADR-001: Stateless LLM as co-processor; CMOS as source of truth

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-002 (memory hierarchy), ADR-003 (two-LLM economy)

## Context

Existing tools (Claude Projects, ChatGPT Memory, Cursor, MemGPT, Letta, etc.) treat the LLM as the entity that "remembers" — they bolt memory features onto chat. This inherits the fundamental limitations of the transformer:

- Stateless API by design (every call starts fresh).
- Effective context << advertised context (RULER benchmark).
- KV-cache cannot transfer between sessions.
- "Memory" is just chat-history-replay or single-vector retrieval.

We need to invert this. The transformer cannot be made into a "remembering" entity without changing its architecture. So we treat it as what it actually is: a pure function `f(context) → tokens`.

## Decision

**LLM is a stateless co-processor. CMOS is the single source of truth for all persistent state — memory, identity, policies, history, relationships, decisions.**

Every interaction with an LLM is conceptually:
1. CMOS assembles a budget-bounded context from its memory.
2. LLM produces tokens.
3. CMOS extracts new facts, validates against policies, persists.
4. CMOS — not the LLM — is what survives across sessions, providers, model upgrades.

## Rationale

- **Architecturally honest.** It matches what transformers are. We stop pretending stateful memory is a model feature.
- **Provider-portable.** When Opus 4.7 is replaced by 4.8 or by Gemini 3, nothing in CMOS changes. Memory is not bound to a vendor.
- **Observability becomes possible.** If LLM is a black box that "remembers somehow," we can never see what it knows. If CMOS owns state, every fact is inspectable in the GUI.
- **Token economics align.** Cloud LLM is the most expensive component per call; minimizing what flows into it is the dominant cost lever (see ADR-003).
- **Wake-up resilience extends to the system itself.** A system whose state lives in named, structured files survives crashes, migrations, and the shutdown of any LLM provider.

## Consequences

### Positive
- All persistence problems become engineering problems on our side, not provider-feature requests.
- We can plug different LLM backends with no architectural change.
- Time Travel Debugging is feasible because we own every input/output.
- Multi-project isolation is straightforward (it's our database to namespace).

### Negative
- We do not benefit from any "automatic memory" features the provider may add later — we explicitly bypass them.
- We carry the operational burden of running a daemon that must be available whenever the user wants to call an LLM.
- Bootstrap cost: a project must be ingested into CMOS before benefits appear.

### Neutral / unknowns
- The relationship between CMOS-owned state and provider-side prompt caching needs careful design (see ADR-003 + token-reduction techniques).

## Alternatives considered

- **LLM-managed memory (MemGPT-style):** model decides what to archive/recall via function calls. Rejected: every memory decision is an inference call (expensive); decisions are unreliable; no project-level cognition; no observability.
- **Provider-native memory (Claude Projects, ChatGPT Memory):** rejected for vendor lock-in, opacity, no programmatic introspection, no cross-project memory, weak schema.
- **Hybrid: LLM-aware of memory, but provider-agnostic:** considered but adds complexity for no architectural benefit. CMOS's job is exactly to hide memory from the LLM behind a uniform context.

## Implementation notes

- All retrieval, scoring, ranking, deduplication happens in CMOS, never in the cloud LLM.
- LLM never receives instructions like "remember this" — CMOS extracts post-hoc.
- The LLM never sees raw chat history older than the current session window; it sees CMOS-curated digests.

## Revisit conditions

Revisit if a provider ships a memory primitive that is:
1. Programmatically inspectable (can dump state).
2. Programmatically writable from outside the chat.
3. Cross-session, cross-thread reliable.
4. Vendor-portable.

All four together are required. Today no provider satisfies any of them.
