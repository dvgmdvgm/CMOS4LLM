# Component: Sub-LM Runtime

> Pool of local models (3B/14B/32B) for bulk cognitive work ([ADR-003](../02-decisions/ADR-003-two-llm-economy.md)).

## Responsibility

- Model lifecycle management (load, unload, hot-swap)
- Task queue with priority levels: live > consolidation > drift > counterfactual
- Task types: classification, extraction, summarization, dedup, lint, drift scan, counterfactual replay
- Fallback to cheap cloud model (Haiku) when no local GPU available

## Runtime Options

TODO: Evaluate llama.cpp vs vLLM vs MLX (ADR-014 pending)
TODO: Define model selection per task type
TODO: Define batching strategy for background tasks

## Performance Contract

- Classification (critical path): ≤30ms single forward pass
- Background tasks: throughput-optimized, no latency SLA

## Dependencies

- Memory Layers (reads L2–L4 for context, writes extracted facts)
- Policy Engine (drift detection results)

## Scope

- [MVP M3](../03-scope/mvp.md) — single model, 5 task types
- [V1.E](../03-scope/v1.md) — multi-model pool, counterfactual workload
- [V3.B](../03-scope/v3.md) — LoRA-as-memory research

## Open Questions

TODO: GPU memory management strategy (owner's hardware TBD)
TODO: Quantization level vs quality tradeoff per task
