# Research Track: KV-Cache Persistence

> [V3.A](../03-scope/v3.md) — Persistent KV-cache infrastructure for cross-session state transfer.

## Hypothesis

If KV-cache state could be serialized, stored, and restored across sessions, CMOS could achieve 10×+ additional token reduction by avoiding re-processing of stable context.

## Key Challenges

TODO: KV-cache is tied to model weights, positions, and inference node
TODO: Quantization/compression of KV states for storage
TODO: Invalidation strategy when underlying facts change
TODO: Provider cooperation requirements (or self-hosted inference)

## Related Work

TODO: Anthropic prompt caching (5min TTL — partial solution)
TODO: vLLM prefix caching
TODO: PagedAttention and its persistence implications
TODO: Academic papers on KV-cache compression

## CMOS Integration Points

- L1 Working memory (would become persistent L1.5)
- Context Hypervisor (cache-aware assembly)
- Token reduction technique #12 in architecture.md

## Dependencies

- Requires self-hosted inference OR provider API support
- Related to [ADR-003](../02-decisions/ADR-003-two-llm-economy.md) Sub-LM runtime

## Status

Not started. Research-grade. Requires partnerships or self-hosted infrastructure.
