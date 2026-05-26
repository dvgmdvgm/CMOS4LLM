# Research Track: External Attention Systems

> [V3.E](../03-scope/v3.md) — Augmenting transformer attention with external, persistent attention mechanisms.

## Hypothesis

If attention patterns could be externalized and persisted, the model could "attend to" information not in the current context window — effectively extending memory without extending the prompt.

## Key Challenges

TODO: External attention mechanism design (cross-attention to external store?)
TODO: Training/adaptation requirements
TODO: Latency impact of external attention lookups
TODO: Compatibility with existing transformer architectures

## Related Work

TODO: RETRO (chunked cross-attention to retrieval database)
TODO: Memorizing Transformers (kNN over past hidden states)
TODO: Longformer / BigBird (sparse attention patterns)
TODO: Ring Attention (distributed context)

## CMOS Integration Points

- Retrieval Router (attention-guided retrieval)
- L4 Project memory (external attention target)
- Sub-LM Runtime (if adapting local models)

## Dependencies

- Requires model architecture modifications or specialized inference
- May require training/fine-tuning pipeline

## Status

Not started. Research-grade. Depends on advances in retrieval-augmented architectures.
