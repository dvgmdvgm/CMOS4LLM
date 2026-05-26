# ADR-005: Time Travel Debugging is in V1, not V2

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner
- **Related:** ADR-002 (memory hierarchy), ADR-008 (counterfactual mode)

## Context

Time Travel Debugging — the ability to open any past inference call, see its full assembled context, and replay/inspect/counterfactual it — is one of CMOS's killer features. It gives developers a level of LLM-call introspection that no existing tool offers.

The cost of putting it in V1 vs. deferring to V2:

- **In V1:** every storage decision in MVP must support deterministic replay. Event-sourced logs, immutable inference records, full retention of assembled prompts. This is a significant constraint on early schema.
- **In V2:** schema is allowed to optimize for "current state" first, and we add event logs later. Cheaper MVP, but retrofitting time-travel into a non-event-sourced system is brutal.

Owner was asked: ship in V1 (more upfront work) or defer (cheaper start, expensive retrofit). Owner chose V1.

## Decision

**Time Travel Debugging is a V1 commitment.** All inference-related state is event-sourced from the first commit. Every inference call records an immutable `InferenceRecord` with: full assembled prompt, full response, retrieved items with relevance scores, active policies, DNA version, timestamp, project_id. These records live in L2/L3 (depending on importance) and graduate to L5 with archival semantics. They are never mutated.

This is a hard schema constraint, not a feature. The MVP cannot ship without it.

## Rationale

- **Owner's explicit choice** — V1 is the binding scope.
- **Schema retrofit is genuinely brutal.** "Just add event log later" sounds easy and isn't. Ordering, foreign keys, replay determinism, retention boundaries all have to be redesigned. Better to design for it from day one.
- **Counterfactual mode (ADR-008) requires time-travel substrate anyway.** The two are siblings; ADR-008 is in V1, so this must be too.
- **Observability mantra of CMOS demands it.** "If LLM is a black box, we can never see what it knows" was the founding argument. Time-travel is the *strongest* observability we can give the user.
- **Debugging "why did the model do X" is the single most painful problem in LLM work.** Time-travel is the cure.

## Consequences

### Positive
- Killer feature available from V1 — the GUI's Live Inference Inspector becomes a true debugger, not just a real-time viewer.
- Counterfactual mode (ADR-008) lands cleanly on top.
- Audit trail for compliance scenarios (V2+) is already in place.
- Developers can finally answer "what changed between turn N and turn N+1 that led to this response?"

### Negative
- L2/L3 size grows faster (every inference is recorded). Storage budget for typical solo project: estimate ~100–500 MB per month of heavy use. Acceptable.
- Schema must be event-sourced from day one — adds initial design weight.
- Privacy implication: full prompts may include code. Retention policy must be configurable (see Q3 in ROADBLOCKS).

### Neutral / unknowns
- Compression of stored inference records (deduplicate repeated prompt prefixes against L4 snapshots) — V1 optimization, not blocker.
- UX for navigating thousands of past inferences in Live Inference Inspector — needs design (see GUI specs).

## Alternatives considered

- **Defer to V2:** rejected by owner. Cheaper MVP, but retrofit pain + loss of counterfactual mode in V1.
- **Partial time-travel (last N turns only):** rejected — arbitrary cutoff, breaks the use case "what did we decide three weeks ago."
- **Optional / opt-in time-travel:** rejected — half the value comes from being able to retrospectively turn it on for any past turn.

## Implementation notes

- `InferenceRecord` schema is part of the core L2 event log, not a sidecar.
- Records are referenced from L3 episodes (an episode = an ordered set of inference records + extracted lessons).
- L4 has a "DNA version snapshot" reference that lets time-travel restore the active DNA at the time of the call.
- Replay determinism: given the same `InferenceRecord` (prompt, retrieved items, model name + version), re-running through the same model should produce the same response (modulo provider non-determinism — record temperature/seed where supported).
- GUI: Live Inference Inspector has a "browse history" mode showing past records with timestamps, project filter, search by query content.

## Revisit conditions

- If storage growth exceeds ~5 GB / month for a solo user, add compression (delta-encode stored prompts against L4 snapshots) and/or retention policy.
- If providers move to deterministic-by-default sampling, replay UX simplifies significantly — current ADR is forward-compatible.
