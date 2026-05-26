# ADR-004: Multi-project from day one

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude

## Context

Owner's primary use case is a Django marketplace, but owner has and will have other projects. Many existing memory tools (Claude Projects, ChatGPT Memory) treat single-project as the primitive and bolt multi-project on later, which leads to leaky memory, ambiguous policies, and identity confusion.

Owner explicitly chose "multi-project from the box" during GUI design discussion.

## Decision

**CMOS treats project as a first-class isolation boundary from the very first commit.** Every memory item, policy, episode, DNA, telemetry record, and trace carries an explicit `project_id`. There is never a "default project" or implicit fallback. The user explicitly selects a project context for any operation.

A single CMOS daemon serves multiple projects. Switching projects is a UI/CLI operation, not a relaunch.

## Rationale

- **Retrofitting multi-tenancy is painful.** Schema changes that thread `project_id` through every table after the fact are expensive and error-prone. Doing it on day one is cheap.
- **Cross-project leakage is the worst memory bug.** A policy from marketplace ("money is Decimal") leaking into a side-project where money is not a concept is confusing. Worse, an architectural decision from project A subtly biasing suggestions in project B is invisible bug.
- **Project DNA only makes sense scoped.** Each project has its own constitution; merging or sharing DNAs is a deliberate cross-project transfer (V2+), not a default.
- **Multi-tenancy from start enables team scenarios later.** When V2 adds shared CMOS instances for small teams, the boundary is already there.

## Consequences

### Positive
- Clean isolation guarantees from day one.
- Project switcher is a real UI affordance (Cognitive Console drop-down).
- Per-project Token Analytics, drift logs, telemetry — all natural.
- Foundation for V2 multi-user (project-level ACLs are the natural unit).

### Negative
- Every API, every query, every storage row carries `project_id` from day one — small constant overhead in code and schema.
- "Where did this fact come from" becomes a per-project question — slightly more bookkeeping in trace UI.
- Encourages premature project segmentation by users who might benefit from sharing context (e.g., monorepo with multiple modules — should they be one project or several?). Documentation must address this.

### Neutral / unknowns
- Cross-project memory transfer (V2+) needs explicit, deliberate UX — not auto.
- Whether to allow "project group" (multiple projects sharing some DNA but not others) is open — defer.

## Alternatives considered

- **Single-project default + multi-project as upgrade:** rejected — retrofit cost.
- **One CMOS instance per project:** rejected — operational burden, no path to multi-project queries, breaks Cognitive Console as a unified surface.
- **Project as tag, not isolation boundary:** rejected — "tags" make leakage easy and isolation hard.

## Implementation notes

- Storage schemas: `project_id` is a non-null column / KG label / partition key everywhere.
- Gateway requires `project_id` in session establishment; refuses requests without one.
- CLI: `cmos --project marketplace memory show ...`. No implicit current project; explicit only.
- Cognitive Console header always shows current project; switching requires a click, not a swipe.
- Bootstrap pipeline creates a project scaffold first, then ingests into it.

## Revisit conditions

- If usage patterns show 99% of users have one project ever, consider ergonomic shortcut (still keep `project_id` in schema).
- When V2 multi-user lands, this ADR may be supplemented with ADR for cross-project ACLs and shared DNAs.
