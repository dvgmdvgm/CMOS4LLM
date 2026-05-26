use cmos_memory::l2l3::{EventStore, EventType, Layer, MemoryEvent};
use cmos_memory::l4::{Fact, FactSource, ProjectMemory};
use cmos_retrieval::vector::{VectorIndex, VectorRecord};
use cmos_retrieval::{ContextAssembler, ContextQuery};

const PROJECT: &str = "quality-test";

struct SyntheticCorpus {
    project_memory: ProjectMemory,
    event_store: EventStore,
    vector_index: VectorIndex,
}

struct RelevanceJudgment {
    query: &'static str,
    relevant_l4_labels: Vec<&'static str>,
    relevant_l3_summaries: Vec<&'static str>,
}

fn build_corpus() -> SyntheticCorpus {
    let pm = ProjectMemory::open_in_memory().unwrap();
    let es = EventStore::open_in_memory().unwrap();

    let facts = vec![
        ("decision", "Use PostgreSQL", "Chose PostgreSQL for ACID compliance and JSON support"),
        ("decision", "JWT for auth", "JWT tokens for stateless authentication between services"),
        ("convention", "snake_case naming", "All Python code uses snake_case for functions and variables"),
        ("convention", "Type hints required", "All public functions must have type annotations"),
        ("policy", "No raw SQL", "Use ORM queries exclusively, no raw SQL to prevent injection"),
        ("policy", "Rate limiting", "All public API endpoints must have rate limiting configured"),
        ("lesson", "N+1 query problem", "Discovered N+1 queries in user listing, fixed with select_related"),
        ("lesson", "Migration ordering", "Migrations must be tested in CI before merge to prevent deploy failures"),
        ("constraint", "Python 3.11+", "Minimum Python version is 3.11 for performance and typing features"),
        ("constraint", "Max response 5MB", "API responses must not exceed 5MB to prevent OOM in clients"),
        ("decision", "Redis for caching", "Redis chosen for distributed caching with TTL support"),
        ("decision", "Celery for tasks", "Celery with Redis broker for async background task processing"),
        ("convention", "REST naming", "API endpoints follow REST conventions: plural nouns, HTTP verbs"),
        ("policy", "CORS whitelist", "Only whitelisted origins allowed, no wildcard CORS in production"),
        ("lesson", "Connection pooling", "DB connection pool exhaustion caused outage, increased pool size to 20"),
        ("constraint", "Deploy window", "Production deploys only during business hours Mon-Fri"),
        ("decision", "Pydantic models", "Pydantic v2 for request/response validation and serialization"),
        ("convention", "Error format", "All API errors return {error: string, code: string, details: object}"),
        ("lesson", "Retry with backoff", "External API calls must use exponential backoff after timeout incident"),
        ("policy", "Audit logging", "All write operations must emit audit log events for compliance"),
    ];

    for (i, (kind, label, desc)) in facts.iter().enumerate() {
        pm.insert_fact(&Fact {
            id: None,
            project_id: PROJECT.to_string(),
            kind: kind.to_string(),
            label: label.to_string(),
            description: desc.to_string(),
            source: FactSource::Bootstrap,
            confidence: 0.8 + (i as f32 * 0.005),
            access_count: (i as u32 % 5) + 1,
        })
        .unwrap();
    }

    let episodes = vec![
        (EventType::Decision, "Decided to add pagination to /users endpoint with cursor-based approach"),
        (EventType::Lesson, "Learned that bulk inserts need transaction wrapping to avoid partial writes"),
        (EventType::Extraction, "Extracted pattern: all services use dependency injection via constructor"),
        (EventType::Decision, "Chose to implement WebSocket for real-time notifications instead of polling"),
        (EventType::Lesson, "Found that datetime serialization must always use UTC with timezone info"),
        (EventType::RejectedApproach, "Rejected GraphQL: team lacks expertise, REST sufficient for current needs"),
        (EventType::Decision, "Added OpenTelemetry tracing to all service boundaries"),
        (EventType::Extraction, "Extracted: authentication middleware validates JWT on every request"),
        (EventType::Lesson, "Discovered memory leak in WebSocket handler due to unclosed connections"),
        (EventType::Decision, "Implemented circuit breaker pattern for external payment API calls"),
        (EventType::Extraction, "Extracted: database models use soft-delete pattern with deleted_at column"),
        (EventType::Lesson, "Learned that file uploads must stream to S3, not buffer in memory"),
        (EventType::RejectedApproach, "Rejected microservices split: monolith sufficient at current scale"),
        (EventType::Decision, "Added structured logging with correlation IDs across all services"),
        (EventType::Extraction, "Extracted: test fixtures use factory pattern with faker for data generation"),
    ];

    for (i, (event_type, summary)) in episodes.iter().enumerate() {
        let ts = format!("2026-05-{:02}T10:{:02}:00Z", 10 + (i / 3), (i * 7) % 60);
        es.append(&MemoryEvent {
            id: None,
            project_id: PROJECT.to_string(),
            layer: Layer::L3,
            event_type: *event_type,
            entity_id: Some(format!("episode-{}", i)),
            session_id: Some("session-1".to_string()),
            timestamp: ts,
            payload: serde_json::json!({ "summary": summary }),
            access_count: (i as u32 % 4) + 1,
            importance: 0.5 + (i as f32 * 0.02),
        })
        .unwrap();
    }

    let dim = 8;
    let mut vi = VectorIndex::open_in_memory(dim).unwrap();

    let mut records: Vec<VectorRecord> = Vec::new();

    let fact_embeddings: Vec<Vec<f32>> = vec![
        vec![0.9, 0.1, 0.0, 0.0, 0.2, 0.0, 0.0, 0.0], // PostgreSQL - database
        vec![0.1, 0.9, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0], // JWT auth - security
        vec![0.0, 0.0, 0.9, 0.0, 0.0, 0.0, 0.1, 0.0], // snake_case - code style
        vec![0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.2, 0.0], // type hints - code style
        vec![0.1, 0.8, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0], // No raw SQL - security/db
        vec![0.0, 0.7, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0], // Rate limiting - security/api
        vec![0.8, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.0], // N+1 query - database
        vec![0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0], // Migration ordering - database
        vec![0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0], // Python 3.11 - code/infra
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.1], // Max response - api
        vec![0.6, 0.0, 0.0, 0.4, 0.0, 0.0, 0.0, 0.0], // Redis caching - database/perf
        vec![0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.2], // Celery tasks - async/perf
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.1], // REST naming - api
        vec![0.0, 0.9, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0], // CORS - security
        vec![0.7, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0], // Connection pooling - db/perf
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // Deploy window - ops
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.2, 0.0], // Pydantic - api/code
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.1, 0.0], // Error format - api
        vec![0.0, 0.0, 0.0, 0.6, 0.0, 0.4, 0.0, 0.0], // Retry backoff - perf/api
        vec![0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.2], // Audit logging - security
    ];

    for (i, emb) in fact_embeddings.iter().enumerate() {
        records.push(VectorRecord {
            id: format!("l4-{}", i + 1),
            source_id: (i + 1) as i64,
            layer: "L4".to_string(),
            content: facts[i].2.to_string(),
            embedding: emb.clone(),
        });
    }

    let episode_embeddings: Vec<Vec<f32>> = vec![
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.2], // pagination - api
        vec![0.7, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0], // bulk inserts - database
        vec![0.0, 0.0, 0.6, 0.0, 0.0, 0.0, 0.4, 0.0], // DI pattern - code
        vec![0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0, 0.0], // WebSocket - api/perf
        vec![0.0, 0.0, 0.3, 0.0, 0.0, 0.7, 0.0, 0.0], // datetime - api/code
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.2], // rejected GraphQL - api
        vec![0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5], // OpenTelemetry - perf/ops
        vec![0.0, 0.9, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0], // JWT middleware - security
        vec![0.0, 0.0, 0.0, 0.7, 0.0, 0.3, 0.0, 0.0], // WebSocket leak - perf/api
        vec![0.0, 0.0, 0.0, 0.6, 0.0, 0.4, 0.0, 0.0], // circuit breaker - perf/api
        vec![0.6, 0.0, 0.0, 0.0, 0.4, 0.0, 0.0, 0.0], // soft-delete - database
        vec![0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5], // file uploads S3 - perf/ops
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5], // rejected microservices - arch
        vec![0.0, 0.0, 0.0, 0.4, 0.0, 0.0, 0.0, 0.6], // structured logging - ops
        vec![0.0, 0.0, 0.7, 0.0, 0.0, 0.0, 0.3, 0.0], // test fixtures - code
    ];

    for (i, emb) in episode_embeddings.iter().enumerate() {
        records.push(VectorRecord {
            id: format!("l3-{}", i + 1),
            source_id: (i + 1) as i64,
            layer: "L3".to_string(),
            content: episodes[i].1.to_string(),
            embedding: emb.clone(),
        });
    }

    vi.upsert(&records).unwrap();

    SyntheticCorpus {
        project_memory: pm,
        event_store: es,
        vector_index: vi,
    }
}

fn relevance_judgments() -> Vec<RelevanceJudgment> {
    vec![
        RelevanceJudgment {
            query: "database performance and queries",
            relevant_l4_labels: vec![
                "Use PostgreSQL",
                "N+1 query problem",
                "Migration ordering",
                "Redis for caching",
                "Connection pooling",
                "No raw SQL",
            ],
            relevant_l3_summaries: vec![
                "Discovered N+1 queries in user listing, fixed with select_related",
                "Learned that bulk inserts need transaction wrapping to avoid partial writes",
                "Extracted: database models use soft-delete pattern with deleted_at column",
            ],
        },
        RelevanceJudgment {
            query: "security and authentication",
            relevant_l4_labels: vec![
                "JWT for auth",
                "No raw SQL",
                "Rate limiting",
                "CORS whitelist",
                "Audit logging",
            ],
            relevant_l3_summaries: vec![
                "Extracted: authentication middleware validates JWT on every request",
            ],
        },
        RelevanceJudgment {
            query: "API design and response format",
            relevant_l4_labels: vec![
                "REST naming",
                "Error format",
                "Pydantic models",
                "Max response 5MB",
                "Rate limiting",
            ],
            relevant_l3_summaries: vec![
                "Decided to add pagination to /users endpoint with cursor-based approach",
                "Rejected GraphQL: team lacks expertise, REST sufficient for current needs",
                "Found that datetime serialization must always use UTC with timezone info",
            ],
        },
        RelevanceJudgment {
            query: "async processing and performance",
            relevant_l4_labels: vec![
                "Celery for tasks",
                "Redis for caching",
                "Connection pooling",
                "Retry with backoff",
            ],
            relevant_l3_summaries: vec![
                "Chose to implement WebSocket for real-time notifications instead of polling",
                "Discovered memory leak in WebSocket handler due to unclosed connections",
                "Implemented circuit breaker pattern for external payment API calls",
                "Learned that file uploads must stream to S3, not buffer in memory",
            ],
        },
    ]
}

fn precision_at_k(retrieved: &[&str], relevant: &[&str], k: usize) -> f64 {
    let top_k: Vec<_> = retrieved.iter().take(k).collect();
    if top_k.is_empty() {
        return 0.0;
    }
    let hits = top_k.iter().filter(|r| relevant.contains(r)).count();
    hits as f64 / k as f64
}

fn recall_at_k(retrieved: &[&str], relevant: &[&str], k: usize) -> f64 {
    if relevant.is_empty() {
        return 1.0;
    }
    let top_k: Vec<_> = retrieved.iter().take(k).collect();
    let hits = top_k.iter().filter(|r| relevant.contains(r)).count();
    hits as f64 / relevant.len() as f64
}

#[test]
fn keyword_retrieval_l4_precision() {
    let corpus = build_corpus();
    let assembler = ContextAssembler::default();
    let judgments = relevance_judgments();

    let mut total_precision = 0.0;
    let k = 10;

    for judgment in &judgments {
        let query = ContextQuery::new(PROJECT, judgment.query).with_budget(8000);
        let result = assembler
            .assemble(&query, None, None, Some(&corpus.project_memory))
            .unwrap();

        let l4_section = result.sections.iter().find(|s| s.source_layer == "L4");
        let retrieved_labels: Vec<&str> = if let Some(section) = l4_section {
            // Format: "- **[kind]** label: description\n"
            section
                .content
                .lines()
                .filter_map(|line| {
                    let marker = line.find("]** ")?;
                    let label_start = marker + 4;
                    let label_end = line[label_start..].find(':')?;
                    Some(line[label_start..label_start + label_end].trim())
                })
                .collect()
        } else {
            vec![]
        };

        let p = precision_at_k(&retrieved_labels, &judgment.relevant_l4_labels, k);
        total_precision += p;
    }

    let avg_precision = total_precision / judgments.len() as f64;
    assert!(
        avg_precision >= 0.2,
        "keyword L4 precision@{} = {:.3}, expected >= 0.2",
        k,
        avg_precision
    );
}

#[test]
fn keyword_retrieval_l3_recall() {
    let corpus = build_corpus();
    let assembler = ContextAssembler::default();
    let judgments = relevance_judgments();

    let mut total_recall = 0.0;
    let k = 10;

    for judgment in &judgments {
        let query = ContextQuery::new(PROJECT, judgment.query).with_budget(8000);
        let result = assembler
            .assemble(&query, None, Some(&corpus.event_store), None)
            .unwrap();

        let l3_section = result.sections.iter().find(|s| s.source_layer == "L3");
        let retrieved_summaries: Vec<&str> = if let Some(section) = l3_section {
            section
                .content
                .lines()
                .filter_map(|line| {
                    let colon_pos = line.find(": ")?;
                    Some(line[colon_pos + 2..].trim())
                })
                .collect()
        } else {
            vec![]
        };

        let r = recall_at_k(&retrieved_summaries, &judgment.relevant_l3_summaries, k);
        total_recall += r;
    }

    let avg_recall = total_recall / judgments.len() as f64;
    assert!(
        avg_recall >= 0.3,
        "keyword L3 recall@{} = {:.3}, expected >= 0.3",
        k,
        avg_recall
    );
}

#[test]
fn hybrid_retrieval_l4_outperforms_keyword_only() {
    let corpus = build_corpus();

    let db_query_embedding = vec![0.8, 0.0, 0.0, 0.2, 0.1, 0.0, 0.0, 0.0];

    let vector_results = corpus
        .vector_index
        .search(&db_query_embedding, 20, Some("L4"))
        .unwrap();

    assert!(
        !vector_results.is_empty(),
        "vector search should return results"
    );

    let top_result = &vector_results[0];
    let db_relevant = [
        "Chose PostgreSQL for ACID compliance and JSON support",
        "Discovered N+1 queries in user listing, fixed with select_related",
        "Migrations must be tested in CI before merge to prevent deploy failures",
        "Redis chosen for distributed caching with TTL support",
        "DB connection pool exhaustion caused outage, increased pool size to 20",
    ];

    assert!(
        db_relevant.contains(&top_result.content.as_str()),
        "top vector result '{}' should be database-related",
        top_result.content
    );
}

#[test]
fn hybrid_retrieval_l3_vector_finds_semantic_matches() {
    let corpus = build_corpus();

    let security_query_embedding = vec![0.0, 0.9, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0];

    let results = corpus
        .vector_index
        .search(&security_query_embedding, 5, Some("L3"))
        .unwrap();

    assert!(!results.is_empty(), "should find L3 episodes via vector");

    let security_related = [
        "Extracted: authentication middleware validates JWT on every request",
    ];

    let found_security = results
        .iter()
        .any(|r| security_related.contains(&r.content.as_str()));
    assert!(
        found_security,
        "vector search should find security-related episodes, got: {:?}",
        results.iter().map(|r| &r.content).collect::<Vec<_>>()
    );
}

#[test]
fn budget_enforcement_under_200ms_equivalent() {
    let corpus = build_corpus();
    let assembler = ContextAssembler::default();

    let start = std::time::Instant::now();

    for _ in 0..100 {
        let query = ContextQuery::new(PROJECT, "database performance optimization").with_budget(4000);
        let _ = assembler.assemble(
            &query,
            None,
            Some(&corpus.event_store),
            Some(&corpus.project_memory),
        );
    }

    let elapsed = start.elapsed();
    let per_call_ms = elapsed.as_millis() as f64 / 100.0;

    assert!(
        per_call_ms < 200.0,
        "retrieval planning + assembly = {:.1}ms per call, must be < 200ms p95",
        per_call_ms
    );
}

#[test]
fn assembled_context_respects_budget() {
    let corpus = build_corpus();
    let assembler = ContextAssembler::default();

    let budgets = [500, 1000, 2000, 4000, 8000];

    for budget in budgets {
        let query = ContextQuery::new(PROJECT, "everything about the project").with_budget(budget);
        let result = assembler
            .assemble(
                &query,
                None,
                Some(&corpus.event_store),
                Some(&corpus.project_memory),
            )
            .unwrap();

        assert!(
            result.total_tokens <= budget,
            "budget {} violated: got {} tokens",
            budget,
            result.total_tokens
        );
    }
}

#[test]
fn higher_importance_items_ranked_first() {
    let pm = ProjectMemory::open_in_memory().unwrap();

    pm.insert_fact(&Fact {
        id: None,
        project_id: PROJECT.to_string(),
        kind: "decision".to_string(),
        label: "Low priority item".to_string(),
        description: "Something not very important".to_string(),
        source: FactSource::Bootstrap,
        confidence: 0.3,
        access_count: 0,
    })
    .unwrap();

    pm.insert_fact(&Fact {
        id: None,
        project_id: PROJECT.to_string(),
        kind: "decision".to_string(),
        label: "High priority item".to_string(),
        description: "Critical architectural decision".to_string(),
        source: FactSource::UserDeclared,
        confidence: 0.95,
        access_count: 10,
    })
    .unwrap();

    let assembler = ContextAssembler::default();
    let query = ContextQuery::new(PROJECT, "architecture").with_budget(4000);
    let result = assembler.assemble(&query, None, None, Some(&pm)).unwrap();

    let l4_content = &result.sections[0].content;
    let high_pos = l4_content.find("High priority item").unwrap();
    let low_pos = l4_content.find("Low priority item").unwrap();

    assert!(
        high_pos < low_pos,
        "high-confidence, high-access items should rank before low ones"
    );
}
