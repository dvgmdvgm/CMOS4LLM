# Component: Bootstrap Pipeline

> Onboards existing projects into CMOS. Populates L4 from source code, git history, and documentation without modifying the target project.

## Responsibility

- Static analysis (AST parsing, Django-specific extractors)
- Domain ontology construction from models/schemas
- Convention mining via Sub-LM (batched)
- Git history mining (commits, blame, PR descriptions)
- Documentation ingestion via Sub-LM
- Interactive policy elicitation (CLI in MVP)

## Pipeline Steps

1. Static AST sweep (no LLM)
2. Schema & domain extraction (no LLM)
3. Architectural pattern detection (no LLM)
4. Convention mining (Sub-LM, batched)
5. Git history mining (no LLM + Sub-LM for summarization)
6. Rejected approaches detection (Sub-LM)
7. Documentation ingestion (Sub-LM)
8. Interactive policy elicitation (human-in-the-loop)

## Performance Target

- <8 hours for 400K LoC Django repo

## Dependencies

- Sub-LM Runtime, Memory Layers (L4 as target), Policy Engine (DNA seeding)

## Scope

- [MVP M1](../03-scope/mvp.md) — Django-aware bootstrap
- [V1.A](../03-scope/v1.md) — framework-agnostic extensions

## Open Questions

TODO: Incremental re-bootstrap strategy (after code changes)
TODO: Parallelism model for large repos
