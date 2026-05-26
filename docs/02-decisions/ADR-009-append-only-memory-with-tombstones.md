# ADR-009: Append-only memory in L4/L5 with tombstones, never hard-delete

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-002 (memory hierarchy), ADR-005 (time travel in V1)

## Context

Project memory in CMOS spans years. Decisions are made, superseded, sometimes reversed. The naïve approach — "update the fact" — destroys precisely the information that makes the memory valuable: *why* things are the way they are, *what was tried before*, *who was wrong about what when*.

Existing systems lose this information silently: when you edit a Cursor rule, the previous version is gone; when ChatGPT Memory updates a fact, you can't see the diff. CMOS exists in part to *reverse* this loss.

## Decision

**L4 and L5 are append-only. Hard-delete is forbidden. Updates are expressed as new versions; deletions are expressed as tombstones.**

Concretely:

- Every fact in L4 has `id`, `version`, `created_at`, `created_by`, `supersedes` (link to previous version, if any).
- "Updating" a fact creates a new version with `supersedes = old_id`. The old version remains, marked superseded.
- "Deleting" a fact creates a tombstone record: `id`, `tombstoned_at`, `tombstoned_by`, `reason`, `subject_id`. The subject record stays.
- Default queries return only current (non-superseded, non-tombstoned) records. Time-travel queries can return historical states.
- L5 archival inherits the same semantics; nothing in L5 is ever physically deleted, though aged records may be compressed (digest replaces full content) — even compression is a versioned operation.
- L1 (working memory) is transient by design — no append-only requirement.
- L2 (session) is event-sourced (every event immutable), but the *materialized views* derived from events can be rebuilt; the events themselves are append-only.
- L3 (episodic) follows L4 semantics for completed episodes.

## Rationale

- **Time travel (ADR-005) requires this.** Without versioned, retained history, you cannot replay or counterfactual past states.
- **Project understanding requires this.** "Why is the code shaped this way?" is a question that demands to see what was tried and rejected, not just what's in place now.
- **Conflict resolution requires this.** When new fact contradicts existing, the resolution is "supersede with version chain," not "overwrite."
- **It's cheap.** Storage is essentially free at the project-fact granularity; the cost is querying — solved by indexing only current versions for hot path.
- **It's psychologically safer.** Owner can edit a fact freely knowing the previous state is recoverable; this encourages active curation rather than reluctant editing.

## Consequences

### Positive
- Project history is queryable.
- DNA versioning, drift trends, evolution timelines all reduce to "query L4 across versions."
- Counterfactual mode (ADR-008) gets its substrate for free.
- Audit trail for compliance (V2+) is automatic.

### Negative
- Storage grows monotonically (slowly — facts are small relative to inference records). Mitigation: compression in L5, but never hard-delete.
- Query patterns must always specify "current only" or "as of timestamp T" — small constant overhead in API.
- Garbage in stays in. A typo'd fact lives forever (corrected via supersession). Mitigation: editorial UX in DNA Editor / Memory Browser surfaces version chains so superseded versions don't pollute working views.

### Neutral / unknowns
- Whether to expose "purge all versions of fact X" as a privileged admin operation (e.g., for accidentally-saved secrets). Default: no. Privacy/compliance scenario (Q3 in ROADBLOCKS) may force a controlled purge mechanism.
- Whether tombstones should be human-deletable after a retention window. Default: no.

## Alternatives considered

- **Mutable + audit log sidecar:** rejected — the audit log is then the real history, and the "current" state can drift from it under bugs. Append-only with tombstones unifies the two.
- **Soft-delete only (no version chain):** rejected — soft-delete tells you a fact existed, but not what came before or after. Insufficient for evolution queries.
- **Full event sourcing for L4:** considered. Equivalent in expressive power but heavier engineering. The fact-level versioning is a pragmatic middle ground.

## Implementation notes

- Schema sketch (L4 fact):
  ```
  id            UUID
  fact_type     enum(rule | decision | fact | rejected | architecture | ...)
  version       int (per id)
  current       bool (false if superseded or tombstoned)
  supersedes    UUID? (the id+version this replaces; null for original)
  superseded_by UUID? (set when later version supersedes this)
  tombstone     bool
  tombstone_reason text?
  created_at    timestamp
  created_by    actor (user / sub-lm / cloud-lm with prompt id)
  payload       jsonb (the fact body)
  evidence_refs UUID[] (decisions, incidents, PRs)
  project_id    UUID (ADR-004)
  ```
- Indexes: `(project_id, fact_type, current)` for hot path; `(project_id, id, version)` for history.
- "Privileged purge" (when implemented for compliance): replaces payload with a redaction marker but preserves `id`, `version`, `created_at`. The version chain remains intact; the content disappears.
- Admin tools surface superseded versions explicitly — they are not invisible, just not in default queries.

## Revisit conditions

- If storage growth becomes a real problem (>100 GB per project), introduce more aggressive compression in L5 (digest-only after N years).
- If a regulatory scenario demands true hard-delete (GDPR right-to-erasure for user-PII inadvertently recorded), introduce the privileged purge mechanism described above. Even then, version chain integrity is preserved.
