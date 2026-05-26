use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cmos_bootstrap::PipelineRunner;

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
    }
}
