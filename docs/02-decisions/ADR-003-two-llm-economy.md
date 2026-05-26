# ADR-003: Two-LLM economy (local Sub-LM + cloud LLM)

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-001 (stateless LLM), ADR-014 (Sub-LM model choice — TBD)

## Context

Today's tools route every chunk of text — extraction, summarization, dedup, linting, drift detection, classification — through the most expensive available LLM (Opus, GPT-5, Gemini Ultra). This is upside-down economics:

- 80%+ of the token volume in any "AI development workflow" is bookkeeping (what changed, what to remember, what to summarize) — work that doesn't need frontier reasoning.
- The work that actually needs frontier reasoning (architectural synthesis, novel code generation, multi-step problem decomposition) is a small fraction.
- Sending bookkeeping to Opus is paying ~$15/M input tokens for work a 14B model on the user's machine could do at zero marginal cost.

## Decision

CMOS adopts a **two-LLM economy**:

- **Sub-LM (local, small):** all bulk cognitive work. Extraction, summarization, fact mining, semantic dedup, drift detection, lint checks, classification, query rewriting, fact validation, episode digestion. Runs on the user's hardware via llama.cpp / vLLM / MLX. Pool of models by size: 3B (classify, rerank, dedup), 14B (extract, summarize, lint), 32B+ coder (AST-aware tasks, complex extraction).
- **Cloud LLM (remote, frontier):** only critical reasoning. Final code generation, architectural decisions, multi-step problem decomposition, anything where the user asked a hard question.

Cloud LLM never receives raw chat, raw files, or raw retrieval output. It receives context that has already been **filtered, compressed, and validated by Sub-LM**.

## Rationale

- **Order-of-magnitude token savings to cloud.** Sub-LM pre-filtering alone is realistically 5–15× reduction in cloud input volume even before other techniques.
- **Latency-friendly for bookkeeping.** Sub-LM runs are batched and asynchronous; they execute in the background between the user's queries. By the time the next query arrives, results are already cached.
- **Privacy-friendly.** Bookkeeping happens locally; sensitive code never leaves the machine for low-value tasks. (See ROADBLOCKS Q3 for cloud-redaction policy.)
- **Compute-frontier-friendly.** As local consumer-grade hardware keeps improving (RTX 50-series, Apple silicon, AMD APUs), Sub-LM capability rises without changing the architecture.
- **Reliability-friendly.** Cloud outages don't kill bookkeeping. CMOS keeps working in degraded "explain-only" mode (Sub-LM only, no cloud).

## Consequences

### Positive
- Drastic cloud cost reduction.
- Locally-controlled rate limits (Sub-LM is unmetered).
- Clear separation of "boring" vs. "expensive" work — easier to reason about cost per feature.
- Background work pipeline (consolidation, drift scans, dedup) runs without polling cloud.

### Negative
- Hard requirement on user hardware: minimum GPU/RAM threshold to run a 14B model at usable speed. Below that, Sub-LM either runs slow or runs CPU-only at unacceptable latency. (Mitigation: fallback to Haiku-class cheap cloud model for "Sub-LM-equivalent" work.)
- Operating two inference stacks (local + cloud) doubles the surface area of "things that can break."
- Sub-LM quality on Django code, on extraction of nuanced facts, on adversarial inputs is empirically unproven for the exact models we'll pick. Requires bench (see Revisit).

### Neutral / unknowns
- Choice of specific Sub-LM model(s) — see ADR-014 (TBD).
- Whether 3B/14B/32B pool actually outperforms a single mid-sized model (e.g., 24B) for our workload — empirical question.

## Alternatives considered

- **Cloud-only:** simplest. Rejected because it fundamentally cannot deliver the token reduction CMOS promises.
- **Local-only:** maximum privacy and zero cloud cost. Rejected because frontier reasoning quality matters for the "hard" 20% of work, and 70B-class models on consumer hardware are too slow for interactive work today.
- **Single mid-sized cloud model (Haiku-class) for bookkeeping + frontier cloud for hard work:** considered as fallback for users without local GPU. Reasonable, but loses privacy benefit and still has metered cost. Will be supported as a fallback profile, not the default.

## Implementation notes

- Sub-LM Runtime is a separate component with its own queue + worker pool.
- Cloud LLM calls go through Hypervisor and are clearly logged distinctly (different cost accounting in Token Analytics).
- Fallback mode (no local GPU): Sub-LM tasks dispatch to a configured cheap cloud model (e.g., Haiku 4.5). User can switch profiles in Settings.
- Sub-LM model swap should be hot-pluggable — the system should not assume a specific model file.
- Privacy boundary: Sub-LM runs a sanitization pass (PII / secrets) on anything destined for cloud (see Q3 in ROADBLOCKS).

## Revisit conditions

- If Sub-LM extraction quality on Django code falls below 80% F1 against a hand-labeled benchmark, revisit model choice (ADR-014).
- If consumer GPUs continue gaining capacity and Sub-LM can run 70B-class models locally at >20 tok/s, the cloud share could shrink further — possibly to "only when explicitly requested."
- If a frontier provider ships a tier specifically priced for bookkeeping (10× cheaper than Opus), the two-LLM split's economics change; reconsider the cloud-only fallback as primary.
