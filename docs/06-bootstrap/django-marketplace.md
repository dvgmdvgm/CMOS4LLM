# Bootstrap Pipeline: Django Marketplace

> Concrete pipeline for onboarding a ~400K LoC Django marketplace into CMOS. Primary validation target for MVP.

## Target Project Profile

- Framework: Django 4.x + DRF
- Size: ~400K LoC Python
- Domain: e-commerce marketplace (users, products, orders, payments, shipping, reviews)
- Infrastructure: Celery, Redis, PostgreSQL, S3

## Pipeline Steps

### Step 1: Static AST Sweep (no LLM)

TODO: Define Python AST extractors (classes, functions, imports, decorators)
TODO: Define Django-specific extractors (models, views, urls, signals, middleware, settings, migrations)
TODO: Define output format (symbol graph nodes + edges)

### Step 2: Schema & Domain Extraction (no LLM)

TODO: Define domain ontology construction from `models.py`
TODO: Define FK/M2M relationship mapping
TODO: Define entity categorization heuristics

### Step 3: Architectural Pattern Detection (no LLM)

TODO: Define layer detection (views → serializers → models → managers)
TODO: Define middleware chain analysis
TODO: Define signal flow mapping
TODO: Define Celery task graph extraction

### Step 4: Convention Mining (Sub-LM, batched)

TODO: Define naming pattern extraction
TODO: Define function size distribution analysis
TODO: Define paradigm detection (CBV vs FBV %, mixin usage)
TODO: Define test layout conventions

### Step 5: Git History Mining

TODO: Define commit log parsing strategy
TODO: Define blame-based ownership mapping
TODO: Define revert/refactor pattern detection
TODO: Define PR description ingestion (if available)

### Step 6: Rejected Approaches Detection (Sub-LM)

TODO: Define deleted-code-in-refactors detection
TODO: Define TODO/FIXME/HACK comment extraction
TODO: Define "removed because" pattern matching

### Step 7: Documentation Ingestion (Sub-LM)

TODO: Define markdown/RST parsing
TODO: Define ADR extraction
TODO: Define CHANGELOG mining

### Step 8: Interactive Policy Elicitation

TODO: Define question set (20–50 questions)
TODO: Define CLI interface for MVP
TODO: Define answer → DNA rule mapping

## Acceptance Criteria ([MVP M1](../03-scope/mvp.md))

- `cmos bootstrap --project marketplace --root <repo>` completes in <8 hours
- L4 Symbol Graph populated and queryable
- Domain ontology contains all major entities
- Initial DNA exists and is human-reviewable

## Open Questions

TODO: Incremental re-bootstrap after code changes
TODO: Parallelism model for steps 1–3 vs 4–7
TODO: Error handling for malformed/unparseable files
