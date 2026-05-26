# GUI Screen: Live Inference Inspector

> Flagship screen. Real-time view of what CMOS assembles, sends, and validates for each inference.

## Purpose

Show the full pipeline for the current/recent inference: what was retrieved, what was excluded, cache hit ratio, streaming response, post-gen validation results.

## Layout

TODO: Define panel layout (likely: left = retrieval plan + items, center = assembled prompt, right = response + validation)
TODO: Define streaming update strategy (WebSocket)

## Key Features

- Real-time assembled context view
- Excluded items with reasons
- Cache hit/miss indicators
- Post-hoc validation results (pass/warn/fail)
- Action buttons: replay, retry-without-X, save-as-episode, explain

## Data Sources

- Context Hypervisor (live inference state)
- Retrieval Router (plan + results)
- Policy Engine (validation results)
- Observability (InferenceRecord)

## Scope

- [MVP M8](../03-scope/mvp.md) — one of 5 MVP GUI screens

## Open Questions

TODO: How to handle multiple concurrent inferences?
TODO: Streaming granularity (token-level vs chunk-level)
