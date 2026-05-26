# Component: Retrieval Router

> Combines multiple retrieval strategies into a parallel plan, executes, deduplicates, and ranks results.

## Responsibility

- Build retrieval plan based on task classification and query analysis
- Execute strategies in parallel: symbol lookup, vector search, graph traversal, temporal, episodic, hybrid
- Deduplicate and rank results by relevance
- Respect token budget constraints from Hypervisor

## Retrieval Strategies

TODO: Define symbol lookup interface (L4 graph queries)
TODO: Define vector search interface (L3/L4 embeddings)
TODO: Define graph traversal patterns (dependency walks, call chains)
TODO: Define temporal retrieval (recent-first, git-aware)
TODO: Define episodic retrieval (similar past tasks from L3)

## Dependencies

- Memory Layers L1–L5, Sub-LM Runtime (for reranking)

## Scope

- [MVP M4](../03-scope/mvp.md) — basic retrieval
- [MVP M5](../03-scope/mvp.md) — lazy loading, differential retrieval

## Open Questions

TODO: Parallel execution model (async tasks vs thread pool)
TODO: Reranking — Sub-LM vs heuristic scoring
