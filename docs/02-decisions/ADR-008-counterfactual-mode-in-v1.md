# ADR-008: Counterfactual mode in V1

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner
- **Related:** ADR-005 (time travel in V1), ADR-002 (memory hierarchy)

## Context

Counterfactual mode is the ability to take past inferences and ask: *"what would have happened if we had a different policy / DNA version / retrieval strategy / model?"* CMOS's Sub-LM re-runs (or re-assembles) those inferences against the alternative configuration and reports differences — verdicts, code diffs, drift impact — so the user can see the empirical effect of a policy change before committing it.

This is a high-cost feature:

- Requires deterministic time-travel substrate (already committed via ADR-005).
- Requires the policy/DNA store to be versioned and queryable at any point in history.
- Requires Sub-LM background workers capable of bulk re-runs without disturbing the live path.
- Adds non-trivial UI surface (a counterfactual setup panel + comparison views).

Owner was asked: ship in V1 (significant work) or defer (cleaner MVP). Owner's reasoning: "если это оправдывает многое" — if it justifies the cost. Owner's decision: ship.

## Decision

**Counterfactual mode lands in V1.**

V1 supports at minimum:

- **Policy counterfactual:** "what changes if I disable / add / modify policy P?" — re-runs the last N inferences in scope.
- **DNA counterfactual:** "what changes if I roll back to DNA version vX?" — uses versioned DNA snapshots to re-assemble the prompt.
- **Retrieval counterfactual:** "what if I had used a different retrieval strategy / different K?" — re-runs assembly only (no LLM call) and shows what would have been included.
- **Comparison view:** side-by-side, original vs counterfactual: assembled context delta, response delta (where re-run with cloud LLM is feasible), drift verdict delta.

Cloud-LLM-execution counterfactuals (re-running with the actual cloud model) are opt-in (cost). Sub-LM-only counterfactuals (assembly + Sub-LM-driven analysis) are the default.

## Rationale

- **Owner's explicit cost-benefit decision.** This is the binding scope.
- **Counterfactual is what turns CMOS from "memory store" into "policy laboratory."** Without it, every policy change is a leap of faith — exactly the problem the system is supposed to solve. With it, "what if I make this rule stricter?" becomes an empirical question with a measured answer.
- **Time-travel substrate (ADR-005) already covers ~80% of the cost.** Once inferences are immutable and versioned, counterfactual is mostly UX + Sub-LM orchestration.
- **It changes the conversation about policy.** Instead of "let's try this rule and see what breaks in production," you measure first, then commit.

## Consequences

### Positive
- Empirical policy management — A/B testing for rules.
- Strong differentiator: no other LLM tool offers this.
- Drift Monitor and DNA Editor become more valuable: "let me run a counterfactual" becomes a one-click escape hatch when a rule's value is unclear.

### Negative
- Sub-LM workload for counterfactual runs is significant (re-running 30 inferences with policy variations is a lot of background compute). Mitigations: bound default scope to "last 30" or "tag-selected" inferences; allow user to widen.
- Cloud counterfactual runs cost money; UI must make this cost visible before launching.
- Implementation complexity in the Hypervisor: re-assembly logic must be parameterized, not hard-coded.

### Neutral / unknowns
- How the user picks "scope" of inferences for counterfactual — by recency, by project area, by tag — V1 starts with "recency + scope tag," extends in V1+.
- Storage cost of counterfactual reports — cap retention to N days, archive on demand.

## Alternatives considered

- **Defer to V2:** rejected by owner.
- **Sub-LM-only counterfactuals (no cloud re-run):** considered as a partial-V1 fallback. We do this *and* keep cloud re-run as opt-in — the partial mode is the default to keep cost predictable.
- **Single-call counterfactual (one inference at a time):** rejected as primary mode — value comes from "across many inferences I see the policy's pattern." Single-call becomes a special case of bulk.

## Implementation notes

- Hypervisor must accept a `CounterfactualConfig` parameter that overrides: active policies (set ID list to add/remove), DNA version, retrieval plan, model. The same code path produces real and counterfactual assemblies.
- Sub-LM Runtime gets a low-priority counterfactual queue — never blocks the live user path.
- Comparison view: in `Live Inference Inspector` add a "Counterfactual ▾" button on past inferences; in `DNA Editor` add a "test this change" button; in `Policy Manager` add a "what would happen?" link per policy.
- Cost meter: each counterfactual run shows "Sub-LM only: free" or "Cloud LLM run: ~$X estimated" before user confirms.

## Revisit conditions

- If empirical use shows users always run cloud counterfactuals (and the cost is a problem), invest in better Sub-LM surrogate models that approximate the cloud LLM's response.
- If counterfactual mode is rarely used, demote to V2 and reclaim engineering capacity (unlikely, but worth tracking).
