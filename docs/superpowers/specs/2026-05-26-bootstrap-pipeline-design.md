# Bootstrap Pipeline — Design Spec

> Full 8-phase pipeline for onboarding existing projects into CMOS L4 graph.
> Primary target: Django marketplace (~391 Python files, 3 apps).

---

## 1. Overview

The Bootstrap Pipeline performs static analysis, LM-assisted extraction, git mining, and interactive policy elicitation to populate the L4 symbol graph for an existing project. It runs as a sequential pipeline with checkpoint/resume support.

**CLI entry point:**
```
cmos bootstrap --project <name> --root <path> [--resume] [--backend ollama|api] [--model <name>] [--no-interactive] [--skip-phases 5,6]
cmos bootstrap status --project <name>
cmos graph query --project <name> --kind <node_kind>
cmos graph stats --project <name>
```

**Key constraints:**
- Sequential execution (phases 1-8 in order)
- Checkpoint after each phase (resume from last completed)
- No silent fallbacks — always report what was skipped and why
- Append-only graph store (no hard deletes, tombstones only)
- Language-agnostic architecture (trait `LanguageExtractor`), Django extractor first

---

## 2. Architecture

```
CLI (clap)
  │
  ▼
PipelineRunner
  ├── loads/creates ProjectConfig (.cmos/config.toml)
  ├── opens L4 GraphStore (SQLite)
  ├── iterates phases 1..8
  ├── writes checkpoint after each phase
  └── reports progress (phase N/8, items processed)
        │
        ▼ (for each phase)
  trait Phase {
      fn name(&self) -> &str;
      fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput>;
      fn depends_on(&self) -> &[PhaseId];
  }
        │
        ▼ (phases 4, 6, 7 use)
  trait InferenceBackend {
      async fn complete(&self, request: CompletionRequest) -> Result<String>;
      async fn classify(&self, text: &str, categories: &[&str]) -> Result<ClassifyResult>;
      async fn health_check(&self) -> Result<BackendStatus>;
  }
  impl: OllamaBackend, ApiBackend, MockBackend (tests)
        │
        ▼ (all phases write to)
  L4 GraphStore (SQLite)
      - nodes (kind, label, file_path, properties_json)
      - edges (source, target, kind, properties_json)
      - append-only with tombstones
```

**PipelineContext** (shared state passed to all phases):
- `project: ProjectConfig`
- `graph: GraphStore` (SQLite connection)
- `inference: Box<dyn InferenceBackend>` (for LM phases)
- `progress: ProgressReporter` (terminal output)
- `root_path: PathBuf` (target project root)

---

## 3. Phases

### Phase 1: Static AST Sweep (no LM)

- **Input:** recursive `**/*.py` from project root
- **Parser:** tree-sitter-python (in-process, Rust crate)
- **Extracts:** classes, functions, imports, decorators, top-level assignments
- **Django-specific detection:**
  - `models.Model` subclasses → kind `django_model`
  - `@api_view` / CBV → kind `django_view`
  - `urlpatterns` list → kind `django_url`
  - `@receiver` decorator → kind `signal_handler`
  - `class Meta` → stored in parent model's properties
- **Output:** nodes in L4 graph

### Phase 2: Schema & Domain Extraction (no LM)

- **Input:** nodes from Phase 1 where kind = `django_model`
- **Extracts:** model fields (type, constraints), ForeignKey targets, ManyToMany targets, Meta options, indexes
- **Builds edges:** `fk_to`, `m2m_to`, `inherits`
- **Domain ontology:** every Django model becomes a domain entity node
- **Output:** edges + enriched node properties

### Phase 3: Architectural Pattern Detection (no LM)

- **Layer detection:** views → serializers → models → managers (via import analysis)
- **Middleware chain:** parse `MIDDLEWARE` list from settings.py
- **Signal flow:** sender → receiver edges from `@receiver` decorators
- **WebSocket consumers:** Django Channels consumer classes
- **Management commands:** entry points in `management/commands/`
- **DRF serializers:** serializer → model relationships
- **Output:** edges (kind: `calls`, `imports`, `signal_connects`, `middleware_chain`, `serializes`)

### Phase 4: Convention Mining (Sub-LM, batched)

- **Collects statistics:** naming patterns, function size distribution, FBV vs CBV ratio, test layout
- **Batches to Ollama:** groups of 10-20 code samples per inference call
- **Prompt template:** "Analyze these code samples and identify naming conventions, architectural patterns, and style preferences. Output structured JSON."
- **Output:** nodes (kind: `convention`, properties: pattern description, confidence score)

### Phase 5: Git History Mining (no LM)

- **Uses:** `git2` crate (libgit2 bindings for Rust)
- **Extracts:**
  - Commit frequency per file (hotspot detection)
  - File churn score (changes/month)
  - Contributors per module
  - Revert pattern detection
  - Recent activity (last 30/90/365 days)
- **Output:** properties added to existing nodes (`churn_score`, `last_modified`, `contributor_count`, `is_hotspot`)

### Phase 6: Rejected Approaches Detection (Sub-LM)

- **Parses:** TODO/FIXME/HACK comments from all Python files
- **Git analysis:** large deletions in recent commits (>50 lines removed = potential rejected approach)
- **Sub-LM classification:** "Why was this code removed? Categorize: refactor, bug_fix, deprecated, rejected_approach, cleanup"
- **Output:** nodes (kind: `rejected_approach` or `tech_debt_marker`, properties: reason, source_ref)

### Phase 7: Documentation Ingestion (Sub-LM)

- **Scans:** README*, docs/, CHANGELOG*, *.md, ADR files
- **Markdown parsing:** extract sections, headings, code blocks
- **Sub-LM extraction:** "Extract architectural decisions, constraints, domain terms, and project rules from this documentation. Output structured JSON."
- **Output:** nodes (kind: `doc_fact`, `domain_term`, `constraint`, `architectural_decision`)

### Phase 8: Interactive Policy Elicitation (CLI, human-in-loop)

- **Generates questions** based on phases 1-7 results:
  - Detected patterns → "Is this intentional?" (e.g., "You use Decimal for money fields — is this a policy?")
  - Ambiguous conventions → "Which do you prefer?" (e.g., "Some views use DRF, some are plain FBV — preference?")
  - Missing documentation → "Can you clarify?" (e.g., "What's the deployment strategy?")
- **20-50 questions**, presented one at a time via stdin/stdout
- **Answer mapping:** each answer → DNA policy rule
- **Skippable:** `--no-interactive` flag or Ctrl+C saves partial progress
- **Output:** nodes (kind: `policy_rule`, properties: question, answer, confidence, source: "owner")

---

## 4. Data Model (SQLite Schema)

```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    config_json TEXT
);

CREATE TABLE nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL REFERENCES projects(id),
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    file_path TEXT,
    line_start INTEGER,
    line_end INTEGER,
    properties_json TEXT NOT NULL DEFAULT '{}',
    phase_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    tombstoned_at TEXT,
    supersedes INTEGER REFERENCES nodes(id)
);

CREATE TABLE edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL REFERENCES projects(id),
    source_id INTEGER NOT NULL REFERENCES nodes(id),
    target_id INTEGER NOT NULL REFERENCES nodes(id),
    kind TEXT NOT NULL,
    properties_json TEXT DEFAULT '{}',
    phase_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    tombstoned_at TEXT
);

CREATE TABLE pipeline_checkpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL REFERENCES projects(id),
    phase_id INTEGER NOT NULL,
    status TEXT NOT NULL,  -- completed, failed, skipped
    started_at TEXT NOT NULL,
    finished_at TEXT,
    stats_json TEXT
);

CREATE INDEX idx_nodes_project_kind ON nodes(project_id, kind) WHERE tombstoned_at IS NULL;
CREATE INDEX idx_nodes_file ON nodes(project_id, file_path) WHERE tombstoned_at IS NULL;
CREATE INDEX idx_edges_source ON edges(source_id, kind) WHERE tombstoned_at IS NULL;
CREATE INDEX idx_edges_target ON edges(target_id, kind) WHERE tombstoned_at IS NULL;
CREATE INDEX idx_checkpoints_project ON pipeline_checkpoints(project_id, phase_id);
```

**Node kinds:** `function`, `class`, `django_model`, `django_view`, `django_url`, `signal_handler`, `middleware`, `management_command`, `serializer`, `convention`, `rejected_approach`, `tech_debt_marker`, `doc_fact`, `domain_term`, `constraint`, `architectural_decision`, `policy_rule`

**Edge kinds:** `calls`, `imports`, `inherits`, `fk_to`, `m2m_to`, `signal_connects`, `middleware_chain`, `serializes`, `defines_url`, `serves_model`, `uses`

**Time-travel query pattern:**
```sql
SELECT * FROM nodes
WHERE project_id = ?
  AND created_at <= ?
  AND (tombstoned_at IS NULL OR tombstoned_at > ?);
```

---

## 5. InferenceBackend

### Trait

```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<String>;
    async fn classify(&self, text: &str, categories: &[&str]) -> Result<ClassifyResult>;
    async fn health_check(&self) -> Result<BackendStatus>;
}

pub struct CompletionRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub struct ClassifyResult {
    pub category: String,
    pub confidence: f32,
    pub reasoning: Option<String>,
}

pub enum BackendStatus {
    Available { model: String, version: String },
    Unavailable { reason: String },
}
```

### OllamaBackend

- HTTP client (reqwest) to `http://localhost:11434/api/generate`
- Configurable model name (default from config, e.g. `gemma2:latest`)
- Sequential requests (Ollama processes one at a time)
- Timeout: 120s per request, 3 retries with exponential backoff
- Batch helper: sends N prompts sequentially, collects results

### ApiBackend

- OpenAI-compatible HTTP API (works with Anthropic, OpenAI, local proxies)
- API key from `CMOS_API_KEY` env var or config
- Configurable model name and endpoint URL
- Rate limiting (configurable requests/minute)

### MockBackend (testing)

- Returns pre-recorded responses from JSON fixtures
- Used in CI where no Ollama/API is available

---

## 6. Configuration

**File:** `.cmos/config.toml` in target project root (created on first `cmos bootstrap`)

```toml
[project]
name = "marketplace"

[inference]
backend = "ollama"              # "ollama" | "api"
model = "gemma2:latest"
endpoint = "http://localhost:11434"

[inference.api_fallback]
enabled = true
provider = "anthropic"          # "anthropic" | "openai"
model = "claude-haiku-4-5-20251001"
# API key: CMOS_API_KEY env var

[bootstrap]
interactive = true
skip_phases = []
```

**Fallback logic:**
1. Try configured backend
2. If unavailable and `api_fallback.enabled` → switch with warning
3. If both unavailable → skip LM phases (4, 6, 7) with warning, run static phases

---

## 7. Error Handling

| Phase type | Error | Behavior |
|---|---|---|
| Static (1-3) | Parse error on single file | Warning + skip file, continue |
| Static (1-3) | Critical (DB write fail) | Checkpoint + exit with error |
| LM (4, 6, 7) | Ollama timeout | Retry 3x → skip batch with warning |
| LM (4, 6, 7) | Backend unavailable | Try fallback → skip phase with warning |
| Git (5) | Not a git repo | Skip phase with info message |
| Git (5) | Corrupt git history | Warning + partial results |
| Interactive (8) | Ctrl+C | Save partial answers, checkpoint |
| Any | Panic/critical | Checkpoint current state, exit with message |

**Principle:** No silent fallbacks. Every skip/fallback is reported to the user with reason.

---

## 8. Progress Reporting

```
[1/8] Static AST Sweep... 391 files
      +-- functions: 1,247
      +-- classes: 189
      +-- django_models: 43
      +-- django_views: 87
      \-- done (2.3s)
[2/8] Schema & Domain Extraction...
      +-- FK relationships: 67
      +-- M2M relationships: 12
      \-- done (0.4s)
[3/8] Architectural Pattern Detection...
      +-- call edges: 2,341
      +-- signal connections: 5
      +-- middleware chain: 12 layers
      \-- done (1.1s)
[4/8] Convention Mining (gemma2:latest)...
      +-- batch 1/12... done
      +-- batch 2/12... done
      ...
      \-- done (4m 23s)
[5/8] Git History Mining...
      +-- commits analyzed: 1,847
      +-- hotspots detected: 23
      \-- done (8.2s)
[6/8] Rejected Approaches Detection...
      +-- TODO/FIXME found: 34
      +-- large deletions: 12
      \-- done (2m 11s)
[7/8] Documentation Ingestion...
      +-- markdown files: 8
      +-- facts extracted: 45
      \-- done (1m 02s)
[8/8] Policy Elicitation (interactive)...
      Question 1/25: ...

Bootstrap complete. L4 graph: 1,892 nodes, 4,567 edges.
```

---

## 9. Crate Structure

```
crates/
  bootstrap/                    # NEW crate
    Cargo.toml                  # deps: tree-sitter, tree-sitter-python, rusqlite,
                                #       git2, reqwest, tokio, serde, toml, clap
    src/
      lib.rs                    # pub mod declarations, PipelineRunner
      config.rs                 # ProjectConfig, TOML parsing
      context.rs                # PipelineContext (shared state)
      graph_store.rs            # SQLite L4 graph CRUD operations
      progress.rs               # ProgressReporter (terminal output)
      phases/
        mod.rs                  # Phase trait + PhaseId enum
        ast_sweep.rs            # Phase 1: tree-sitter parsing
        schema.rs               # Phase 2: domain extraction
        patterns.rs             # Phase 3: architectural patterns
        conventions.rs          # Phase 4: convention mining (LM)
        git_mining.rs           # Phase 5: git2 history analysis
        rejected.rs             # Phase 6: rejected approaches (LM)
        docs.rs                 # Phase 7: documentation ingestion (LM)
        elicitation.rs          # Phase 8: interactive CLI questionnaire
      extractors/
        mod.rs                  # LanguageExtractor trait
        python.rs               # tree-sitter Python visitor
        django.rs               # Django-specific pattern matching
      inference/
        mod.rs                  # InferenceBackend trait + types
        ollama.rs               # Ollama HTTP client
        api.rs                  # OpenAI-compatible API client
        mock.rs                 # Test mock backend
  cli/                          # EXISTING — add `bootstrap` subcommand
    src/main.rs                 # add BootstrapCmd, GraphCmd
```

---

## 10. Testing Strategy

### Unit Tests
- Each extractor tested on fixture files in `crates/bootstrap/tests/fixtures/`
- Fixtures: `django_models.py`, `django_views.py`, `django_urls.py`, `django_signals.py`, `settings.py`
- Assert: correct nodes/edges created with expected kinds and properties

### Integration Test
- Full pipeline on synthetic mini-Django project (`tests/fixtures/mini_django/`)
- 5-10 files covering all extractable patterns
- Runs phases 1-3 + 5 (static + git), mocks phases 4, 6, 7
- Asserts: graph stats match expected counts

### Snapshot Test
- Run on real project (`D:\art_network_antigravity`)
- Save graph stats as baseline JSON
- On extractor changes: diff shows what changed (regression detection)

### MockBackend for CI
- Pre-recorded LM responses in `tests/fixtures/inference_responses.json`
- Phases 4, 6, 7 use MockBackend in test mode
- Deterministic, no network dependency

---

## 11. Multi-Language Support (Architecture)

The pipeline is designed language-agnostic via the `LanguageExtractor` trait:

```rust
pub trait LanguageExtractor: Send + Sync {
    fn language(&self) -> &str;
    fn file_extensions(&self) -> &[&str];
    fn extract_symbols(&self, source: &[u8], path: &Path) -> Result<Vec<RawNode>>;
    fn extract_relationships(&self, nodes: &[RawNode]) -> Result<Vec<RawEdge>>;
}
```

For MVP: `DjangoExtractor` (implements `LanguageExtractor` for Python/Django).
Future: `TypeScriptExtractor`, `GoExtractor`, `RustExtractor` — same pipeline, different extractors.

---

## 12. Decisions Made

| Decision | Rationale |
|---|---|
| tree-sitter-python (in-process) | Fast, zero-copy, same mechanism for all languages via grammar swap |
| Sequential pipeline | Simpler debugging, Ollama is sequential anyway, checkpoints solve duration |
| Ollama as primary Sub-LM | Already installed on owner's machine with Gemma2 |
| Configurable model + API fallback | Owner wants flexibility to switch models and backends |
| SQLite for L4 graph | ADR-012: embedded, battle-tested, recursive CTEs for traversal |
| Append-only with tombstones | ADR-009: time-travel debugging, no data loss |
| All 8 phases in M1 | Owner decision: full bootstrap in one milestone |

---

## 13. Dependencies (Cargo)

```toml
[dependencies]
tree-sitter = "0.24"
tree-sitter-python = "0.23"
rusqlite = { version = "0.32", features = ["bundled"] }
git2 = "0.19"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
clap = { version = "4", features = ["derive"] }
async-trait = "0.1"
thiserror = "2"
indicatif = "0.17"          # progress bars
walkdir = "2"               # recursive file traversal
```

---

## 14. Acceptance Criteria

1. `cmos bootstrap --project marketplace --root D:\art_network_antigravity` completes all 8 phases
2. L4 graph contains nodes for all Django models, views, URLs, signals from the target project
3. Edges correctly represent FK/M2M/inheritance/call relationships
4. `--resume` correctly skips completed phases
5. `--no-interactive` skips phase 8
6. Ollama unavailable → graceful fallback or skip with warning
7. `cmos graph stats --project marketplace` shows node/edge counts by kind
8. Unit tests pass with fixture files
9. Integration test passes with mini-Django project
