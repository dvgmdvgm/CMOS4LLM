# Component: Policy & Invariant Engine

> Three-tier policy enforcement: suggestions, soft invariants, hard invariants. Symbolic evaluation, not LLM-based.

## Responsibility

- Store and version policies as structured objects
- Pre-flight: inject relevant policies into prompt
- Post-hoc: validate LLM response against soft/hard invariants
- Hard invariant violation → repair loop or block
- Drift detection (background, via Sub-LM)

## Policy Structure

TODO: Define policy schema (id, scope, tier, predicate, rationale, evidence_refs)
TODO: Define predicate language (regex? AST patterns? custom DSL?)
TODO: Define scope resolution (file-level, module-level, project-level)

## Project DNA

TODO: Define DNA schema and injection strategy
TODO: Define DNA versioning and diff format
TODO: Define token budget for DNA (5K–20K target)

## Scope

- [MVP M6](../03-scope/mvp.md) — soft/hard policies, DNA store, post-hoc validation
- [V1.D](../03-scope/v1.md) — constraint hoisting, constrained decoding prep
- [V2.A](../03-scope/v2.md) — full constrained decoding

## Open Questions

TODO: Predicate language design
TODO: Repair loop strategy (re-prompt vs patch)
