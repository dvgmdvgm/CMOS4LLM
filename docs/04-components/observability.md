# Component: Observability & Telemetry

> First-class observability. Every inference is an immutable event. Enables Time Travel Debugging and Counterfactual mode.

## Responsibility

- Record every inference as immutable `InferenceRecord` ([ADR-005](../02-decisions/ADR-005-time-travel-in-v1.md))
- Provide replay/time-travel API for past inferences
- Support counterfactual re-execution ([ADR-008](../02-decisions/ADR-008-counterfactual-mode-in-v1.md))
- Token usage tracking and analytics
- Drift event recording

## InferenceRecord Schema

TODO: Define full schema (see architecture.md §7 for draft)
TODO: Define storage format and indexing strategy
TODO: Define retention policy (estimated 100–500 MB/month)

## Time Travel API

TODO: Define query interface (by time range, by task type, by policy)
TODO: Define replay semantics (same context vs current context)
TODO: Define diff view between original and replay

## Scope

- [MVP M9](../03-scope/mvp.md) — immutable records, replay button
- [V1.F](../03-scope/v1.md) — counterfactual mode, comparison views
- [MVP M8](../03-scope/mvp.md) — Token Analytics GUI screen

## Open Questions

TODO: Storage growth management (compaction? sampling?)
TODO: Counterfactual cost controls (Sub-LM default, cloud opt-in)
