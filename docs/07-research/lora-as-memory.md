# Research Track: LoRA as Memory

> [V3.B](../03-scope/v3.md) — Using LoRA adapters as a form of persistent, trainable memory.

## Hypothesis

Fine-tuned LoRA adapters on project-specific data could encode project knowledge directly into model weights, enabling "memory" that doesn't consume context tokens.

## Key Challenges

TODO: Training data curation from CMOS memory layers
TODO: Catastrophic forgetting when updating adapters
TODO: Adapter composition (multiple projects = multiple LoRAs?)
TODO: Evaluation methodology (how to measure "remembers correctly")

## Related Work

TODO: LoRA/QLoRA fine-tuning literature
TODO: Continual learning research
TODO: Adapter fusion / mixing techniques
TODO: Task-specific adapter routing

## CMOS Integration Points

- Sub-LM Runtime (LoRA loading/switching per project)
- L4 Project memory (training data source)
- Multi-project isolation ([ADR-004](../02-decisions/ADR-004-multi-project-from-day-one.md))

## Dependencies

- Requires Sub-LM runtime with LoRA support (llama.cpp supports this)
- Requires training pipeline (likely offline/batch)

## Status

Not started. Feasible with current tooling but quality/reliability unknown.
