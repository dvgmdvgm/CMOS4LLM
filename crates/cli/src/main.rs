use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cmos_bootstrap::PipelineRunner;
use cmos_memory::{EventStore, ProjectMemory, WorkingMemory};
use cmos_memory::l1::WorkingMemoryConfig;
use cmos_retrieval::{ContextAssembler, ContextQuery, EmbeddingClient, EmbeddingConfig, VectorIndex, VectorRecord};
use cmos_gateway::start_mcp_server;

#[derive(Parser)]
#[command(name = "cmos", about = "Cognitive Memory Operating System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version and verify daemon is operational
    Hello,

    /// Bootstrap a project into the L4 knowledge graph
    Bootstrap {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,

        /// Resume from last checkpoint
        #[arg(long, default_value_t = false)]
        resume: bool,

        /// Inference backend override (ollama or api)
        #[arg(long)]
        backend: Option<String>,

        /// Model name override
        #[arg(long)]
        model: Option<String>,

        /// Disable interactive policy elicitation
        #[arg(long, default_value_t = false)]
        no_interactive: bool,

        /// Phases to skip (comma-separated, e.g. "5,6")
        #[arg(long, value_delimiter = ',')]
        skip_phases: Vec<u8>,
    },

    /// Query the L4 knowledge graph
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },

    /// Assemble context from memory layers for a task
    Context {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,

        /// Task description to assemble context for
        #[arg(long)]
        task: String,

        /// Token budget (default: 32000)
        #[arg(long, default_value_t = 32_000)]
        budget: usize,

        /// Current session ID (for L3 filtering)
        #[arg(long)]
        session: Option<String>,
    },

    /// Inspect memory layer statistics and contents
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },

    /// Manage the vector index for semantic retrieval
    Vector {
        #[command(subcommand)]
        action: VectorAction,
    },

    /// Start the MCP server (stdio transport)
    Mcp {
        /// Path to project data root (contains .cmos/)
        #[arg(long)]
        root: PathBuf,
    },
}

#[derive(Subcommand)]
enum GraphAction {
    /// Show node/edge statistics
    Stats {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root (to find .cmos/graph.db)
        #[arg(long)]
        root: PathBuf,
    },

    /// Query nodes by kind
    Query {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,

        /// Node kind to filter
        #[arg(long)]
        kind: String,
    },
}

#[derive(Subcommand)]
enum MemoryAction {
    /// Show statistics for all memory layers
    Stats {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,
    },

    /// Query events from L2/L3 event store
    Query {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,

        /// Filter by layer (L2 or L3)
        #[arg(long)]
        layer: Option<String>,

        /// Filter by event type
        #[arg(long, name = "type")]
        event_type: Option<String>,

        /// Maximum results to show
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },

    /// Manually trigger promotion engine
    Promote {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,
    },
}

#[derive(Subcommand)]
enum VectorAction {
    /// Index all L3/L4 memory into the vector store
    Index {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,

        /// Embedding model (default: nomic-embed-text:latest)
        #[arg(long, default_value = "nomic-embed-text:latest")]
        model: String,

        /// Ollama endpoint
        #[arg(long, default_value = "http://localhost:11434")]
        endpoint: String,
    },

    /// Search the vector index
    Search {
        /// Project name
        #[arg(long)]
        project: String,

        /// Path to project root
        #[arg(long)]
        root: PathBuf,

        /// Search query
        #[arg(long)]
        query: String,

        /// Maximum results
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// Filter by layer (L3 or L4)
        #[arg(long)]
        layer: Option<String>,

        /// Embedding model
        #[arg(long, default_value = "nomic-embed-text:latest")]
        model: String,

        /// Ollama endpoint
        #[arg(long, default_value = "http://localhost:11434")]
        endpoint: String,
    },

    /// Show vector index statistics
    Stats {
        /// Path to project root
        #[arg(long)]
        root: PathBuf,
    },
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Hello => {
            println!("CMOS v{} — Cognitive Memory Operating System", cmos_core::version());
            println!("Daemon: ready");
        }
        Commands::Bootstrap {
            project,
            root,
            resume,
            backend,
            model,
            no_interactive,
            skip_phases,
        } => {
            let mut runner = PipelineRunner::new(&project, root);
            runner.resume = resume;
            runner.no_interactive = no_interactive;
            runner.skip_phases = skip_phases;
            runner.backend_override = backend;
            runner.model_override = model;

            if let Err(e) = runner.run() {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Graph { action } => match action {
            GraphAction::Stats { project, root } => {
                let db_path = root.join(".cmos").join("graph.db");
                let graph = match cmos_bootstrap::GraphStore::open(&db_path) {
                    Ok(g) => g,
                    Err(e) => {
                        eprintln!("ERROR: cannot open graph: {}", e);
                        std::process::exit(1);
                    }
                };

                println!("=== L4 Graph Stats: {} ===\n", project);

                println!("Nodes by kind:");
                match graph.count_nodes_by_kind(&project) {
                    Ok(counts) => {
                        for (kind, count) in &counts {
                            println!("  {:30} {}", kind, count);
                        }
                        let total: i64 = counts.iter().map(|(_, c)| c).sum();
                        println!("  {:30} {}", "TOTAL", total);
                    }
                    Err(e) => eprintln!("  error: {}", e),
                }

                println!("\nEdges by kind:");
                match graph.count_edges_by_kind(&project) {
                    Ok(counts) => {
                        for (kind, count) in &counts {
                            println!("  {:30} {}", kind, count);
                        }
                        let total: i64 = counts.iter().map(|(_, c)| c).sum();
                        println!("  {:30} {}", "TOTAL", total);
                    }
                    Err(e) => eprintln!("  error: {}", e),
                }
            }
            GraphAction::Query { project, root, kind } => {
                let db_path = root.join(".cmos").join("graph.db");
                let graph = match cmos_bootstrap::GraphStore::open(&db_path) {
                    Ok(g) => g,
                    Err(e) => {
                        eprintln!("ERROR: cannot open graph: {}", e);
                        std::process::exit(1);
                    }
                };

                match graph.query_nodes_by_kind(&project, &kind) {
                    Ok(nodes) => {
                        println!("=== {} nodes (kind: {}) ===\n", nodes.len(), kind);
                        for node in &nodes {
                            let file_info = node.file_path.as_deref().unwrap_or("-");
                            let line_info = node.line_start
                                .map(|l| format!(":{}", l))
                                .unwrap_or_default();
                            println!("  {} ({}{})", node.label, file_info, line_info);
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Context { project, root, task, budget, session } => {
            let events_path = root.join(".cmos").join("events.db");
            let facts_path = root.join(".cmos").join("facts.db");

            let event_store = EventStore::open(&events_path).ok();
            let project_memory = ProjectMemory::open(&facts_path).ok();
            let working_memory = WorkingMemory::new(WorkingMemoryConfig::default());

            let mut query = ContextQuery::new(&project, &task).with_budget(budget);
            if let Some(sid) = session {
                query = query.with_session(&sid);
            }

            let assembler = ContextAssembler::default();
            match assembler.assemble(
                &query,
                Some(&working_memory),
                event_store.as_ref(),
                project_memory.as_ref(),
            ) {
                Ok(ctx) => {
                    println!("{}", ctx.render_with_header(&task));
                }
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Memory { action } => match action {
            MemoryAction::Stats { project, root } => {
                let events_path = root.join(".cmos").join("events.db");
                let facts_path = root.join(".cmos").join("facts.db");

                println!("=== Memory Stats: {} ===\n", project);

                println!("L2/L3 Event Store ({}):", events_path.display());
                match EventStore::open(&events_path) {
                    Ok(es) => {
                        let l2 = es.query_by_layer(&project, cmos_memory::Layer::L2)
                            .map(|v| v.len()).unwrap_or(0);
                        let l3 = es.query_by_layer(&project, cmos_memory::Layer::L3)
                            .map(|v| v.len()).unwrap_or(0);
                        println!("  L2 events: {}", l2);
                        println!("  L3 events: {}", l3);
                        println!("  Total:     {}", l2 + l3);
                    }
                    Err(e) => println!("  (not available: {})", e),
                }

                println!("\nL4 Project Memory ({}):", facts_path.display());
                match ProjectMemory::open(&facts_path) {
                    Ok(pm) => {
                        match pm.count_by_kind(&project) {
                            Ok(counts) => {
                                let mut total = 0i64;
                                for (kind, count) in &counts {
                                    println!("  {:20} {}", kind, count);
                                    total += count;
                                }
                                println!("  {:20} {}", "TOTAL", total);
                            }
                            Err(e) => println!("  error: {}", e),
                        }
                    }
                    Err(e) => println!("  (not available: {})", e),
                }
            }
            MemoryAction::Query { project, root, layer, event_type, limit } => {
                let events_path = root.join(".cmos").join("events.db");
                let es = match EventStore::open(&events_path) {
                    Ok(es) => es,
                    Err(e) => {
                        eprintln!("ERROR: cannot open event store: {}", e);
                        std::process::exit(1);
                    }
                };

                let events = if let Some(layer_str) = &layer {
                    let l = match layer_str.as_str() {
                        "L2" | "l2" => cmos_memory::Layer::L2,
                        "L3" | "l3" => cmos_memory::Layer::L3,
                        _ => {
                            eprintln!("ERROR: invalid layer '{}', use L2 or L3", layer_str);
                            std::process::exit(1);
                        }
                    };
                    es.query_by_layer(&project, l)
                } else {
                    let mut all = es.query_by_layer(&project, cmos_memory::Layer::L2).unwrap_or_default();
                    all.extend(es.query_by_layer(&project, cmos_memory::Layer::L3).unwrap_or_default());
                    Ok(all)
                };

                match events {
                    Ok(mut evts) => {
                        if let Some(et) = &event_type {
                            evts.retain(|e| format!("{:?}", e.event_type).to_lowercase().contains(&et.to_lowercase()));
                        }
                        evts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        let display_count = evts.len().min(limit);
                        println!("=== Events ({} of {}) ===\n", display_count, evts.len());
                        for event in evts.iter().take(limit) {
                            println!("[{:?}] {} | {:?} | importance={:.2}",
                                event.layer, event.timestamp, event.event_type, event.importance);
                            if let Some(summary) = event.payload.get("summary").and_then(|v| v.as_str()) {
                                println!("  {}", summary);
                            }
                            println!();
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            MemoryAction::Promote { project, root } => {
                let events_path = root.join(".cmos").join("events.db");
                let facts_path = root.join(".cmos").join("facts.db");

                let es = match EventStore::open(&events_path) {
                    Ok(es) => es,
                    Err(e) => {
                        eprintln!("ERROR: cannot open event store: {}", e);
                        std::process::exit(1);
                    }
                };
                let pm = match ProjectMemory::open(&facts_path) {
                    Ok(pm) => pm,
                    Err(e) => {
                        eprintln!("ERROR: cannot open project memory: {}", e);
                        std::process::exit(1);
                    }
                };

                let engine = cmos_memory::PromotionEngine::new(Default::default());

                match engine.run_l2_to_l3(&es, &project) {
                    Ok(promoted) => println!("L2→L3: {} events promoted", promoted.len()),
                    Err(e) => eprintln!("L2→L3 error: {}", e),
                }

                match engine.run_l3_to_l4(&es, &pm, &project) {
                    Ok(promoted) => println!("L3→L4: {} facts created", promoted.len()),
                    Err(e) => eprintln!("L3→L4 error: {}", e),
                }

                println!("\nPromotion complete.");
            }
        },
        Commands::Mcp { root } => {
            let data_root = root.join(".cmos");
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(start_mcp_server(data_root)) {
                eprintln!("MCP server error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Vector { action } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match action {
                VectorAction::Index { project, root, model, endpoint } => {
                    let vector_path = root.join(".cmos").join("vectors");
                    let events_path = root.join(".cmos").join("events.db");
                    let facts_path = root.join(".cmos").join("facts.db");

                    let embed_config = EmbeddingConfig {
                        endpoint,
                        model: model.clone(),
                        timeout_secs: 60,
                    };
                    let embed_client = EmbeddingClient::new(embed_config.clone());
                    let dimension = embed_client.dimension();

                    let index = match VectorIndex::open(&vector_path, dimension) {
                        Ok(i) => i,
                        Err(e) => {
                            eprintln!("ERROR: cannot open vector index: {}", e);
                            std::process::exit(1);
                        }
                    };

                    println!("Indexing memory for project '{}' (model: {}, dim: {})", project, model, dimension);

                    let mut records: Vec<VectorRecord> = Vec::new();

                    if let Ok(pm) = ProjectMemory::open(&facts_path) {
                        let kinds = ["decision", "policy", "convention", "lesson", "constraint"];
                        for kind in &kinds {
                            if let Ok(facts) = pm.query_by_kind(&project, kind) {
                                for fact in &facts {
                                    let text = format!("{}: {}", fact.label, fact.description);
                                    records.push(VectorRecord {
                                        id: format!("l4-{}", fact.id.unwrap_or(0)),
                                        source_id: fact.id.unwrap_or(0),
                                        layer: "L4".to_string(),
                                        content: text,
                                        embedding: Vec::new(),
                                    });
                                }
                            }
                        }
                    }

                    if let Ok(es) = EventStore::open(&events_path)
                        && let Ok(events) = es.query_by_layer(&project, cmos_memory::Layer::L3)
                    {
                        for event in &events {
                                let content = if let Some(s) = event.payload.get("summary").and_then(|v| v.as_str()) {
                                    s.to_string()
                                } else if let Some(c) = event.payload.get("content").and_then(|v| v.as_str()) {
                                    c.to_string()
                                } else {
                                    continue;
                                };
                                records.push(VectorRecord {
                                    id: format!("l3-{}", event.id.unwrap_or(0)),
                                    source_id: event.id.unwrap_or(0),
                                    layer: "L3".to_string(),
                                    content,
                                    embedding: Vec::new(),
                                });
                            }
                    }

                    if records.is_empty() {
                        println!("No memory items to index.");
                        return;
                    }

                    println!("Generating embeddings for {} items...", records.len());

                    let batch_size = 32;
                    for chunk in records.chunks_mut(batch_size) {
                        let texts: Vec<String> = chunk.iter().map(|r| r.content.clone()).collect();
                        match rt.block_on(embed_client.embed_batch(&texts)) {
                            Ok(embeddings) => {
                                for (record, emb) in chunk.iter_mut().zip(embeddings) {
                                    record.embedding = emb;
                                }
                            }
                            Err(e) => {
                                eprintln!("ERROR: embedding failed: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }

                    match index.upsert(&records) {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("ERROR: upsert failed: {}", e);
                            std::process::exit(1);
                        }
                    }

                    if let Err(e) = index.save(&vector_path) {
                        eprintln!("WARNING: could not persist index: {}", e);
                    }

                    println!("Indexed {} items into vector store.", records.len());
                }
                VectorAction::Search { project: _, root, query, limit, layer, model, endpoint } => {
                    let vector_path = root.join(".cmos").join("vectors");

                    let embed_config = EmbeddingConfig {
                        endpoint,
                        model,
                        timeout_secs: 30,
                    };
                    let embed_client = EmbeddingClient::new(embed_config);
                    let dimension = embed_client.dimension();

                    let index = match VectorIndex::open(&vector_path, dimension) {
                        Ok(i) => i,
                        Err(e) => {
                            eprintln!("ERROR: cannot open vector index: {}", e);
                            std::process::exit(1);
                        }
                    };

                    let query_embedding = match rt.block_on(embed_client.embed_single(&query)) {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("ERROR: embedding query failed: {}", e);
                            std::process::exit(1);
                        }
                    };

                    let layer_filter = layer.as_deref();
                    match index.search(&query_embedding, limit, layer_filter) {
                        Ok(results) => {
                            println!("=== Vector Search: {} results ===\n", results.len());
                            for (i, r) in results.iter().enumerate() {
                                println!("{}. [{}] (dist: {:.4}) {}", i + 1, r.layer, r.distance, r.content);
                            }
                        }
                        Err(e) => {
                            eprintln!("ERROR: search failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                VectorAction::Stats { root } => {
                    let vector_path = root.join(".cmos").join("vectors");
                    match VectorIndex::open(&vector_path, 768) {
                        Ok(index) => {
                            println!("=== Vector Index Stats ===");
                            println!("  Path:    {}", vector_path.display());
                            println!("  Vectors: {}", index.count());
                        }
                        Err(e) => {
                            eprintln!("ERROR: cannot open vector index: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        },
    }
}
