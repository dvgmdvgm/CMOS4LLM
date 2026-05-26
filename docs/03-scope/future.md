# Scope: Future — Deferred Indefinitely

> Items here are real possibilities, but **not currently scoped to any phase**. Inclusion here is permission to think about them; exclusion from V1/V2/V3 is intentional.
>
> Promotion from Future to a phase requires a deliberate ADR — not "let's just add this."

---

## Future items

### F.1. Mobile UI

- iOS / Android companion app for read-only inspection: dashboard summary, recent inferences, drift alerts, episode notifications.
- Push-notification integration (drift events, completion of long-running counterfactuals).
- Why deferred: mobile is a large platform investment; primary use case (active development) is desktop-bound.

### F.2. Standalone SaaS hosting

- Multi-tenant CMOS-as-a-service.
- Operational complexity (security, isolation, billing, support) is enormous.
- Why deferred: V2 multi-user is single-tenant team mode; SaaS is a separate product decision.

### F.3. Marketplace for project DNA templates

- Public library of starter DNAs for common stacks (Django, FastAPI, Next.js, NestJS, Rails, Laravel, etc.).
- Templates contain hard invariants and forbidden patterns curated from community wisdom.
- Why deferred: V2 introduces cross-project transfer (V2.E); marketplace requires curation policies, governance, possibly moderation.

### F.4. Plugin SDK for third-party retrieval strategies

- Public SDK letting third parties contribute new retrieval strategies (e.g., Tree-sitter-based code retrieval, custom domain retrievers).
- Loaded as plugins by Retrieval Router.
- Why deferred: prerequisite is a stable internal Retrieval Router API, which won't crystallize until V2.I (self-optimization) reveals which extension points matter.

### F.5. Federated memory

- Multiple CMOS instances synchronizing a shared subset of memory across organizations.
- Use cases: open-source project DNA hosted on a public CMOS, privately-edited locally; consortium projects.
- Why deferred: requires solid versioning, conflict resolution, identity model — all evolving through V2/V3.

### F.6. Voice / conversational interface

- Spoken queries to CMOS, spoken summaries.
- Why deferred: orthogonal to core value; voice belongs to GUI client, not substrate.

### F.7. Continuous learning across the user's career

- A "personal CMOS" persisting across all the user's projects forever — career-scale memory.
- Why deferred: privacy, data-portability, retention semantics — all need careful design before opening.

### F.8. Adversarial robustness and prompt-injection defense

- Hardening CMOS against malicious documentation, malicious git history, intentional drift to corrupt project DNA.
- Why deferred: not needed at V1/V2 (single-user / small team trusted scenarios); critical when going SaaS or federated.

### F.9. Provider-agnostic compliance / region pinning

- Route Sub-LM / cloud calls based on regulatory region; prove data-residency.
- Why deferred: relevant to enterprise / SaaS — not solo or small-team scenarios.

### F.10. Integration with project management tools

- Jira / Linear / GitHub Issues bidirectional sync: episodes ↔ tickets.
- Why deferred: every PM tool is its own integration; should be plugins (F.4) once SDK lands.

---

## How to promote a Future item

1. Open an ADR proposing inclusion in a specific phase (V2 / V3).
2. Justify against the phase's goal.
3. Identify dependencies and what they unblock.
4. Owner decides; if accepted, the item moves to the phase's scope file and is removed from Future.

Without that process, Future items stay deferred — including under pressure to "just add it real quick."
