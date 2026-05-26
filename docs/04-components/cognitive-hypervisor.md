# Component: Context Hypervisor

> Central orchestrator. Receives classified requests, plans retrieval, assembles context, enforces token budget, dispatches to cloud LLM, post-processes responses.

## Responsibility

- Task classification (delegates to Sub-LM)
- Retrieval planning (delegates to Retrieval Router)
- Prompt assembly with token budget enforcement
- Cloud LLM dispatch and response streaming
- Post-hoc validation (delegates to Policy Engine)
- Fact extraction trigger (delegates to Sub-LM, background)

## Latency Contract

- Total pre-LLM critical path: <200ms p95
- Max 1 Sub-LM call on critical path (classification)

## Dependencies

- Retrieval Router, Policy Engine, Sub-LM Runtime, L1 Memory, Gateway

## Scope

- [MVP M4](../03-scope/mvp.md) — core hypervisor
- [V1.C](../03-scope/v1.md) — compressed cognition, semantic delta

## Open Questions

TODO: Define task classification taxonomy
TODO: Token budget allocation strategy across DNA / policies / retrieved / user query
TODO: Streaming vs batch response handling
