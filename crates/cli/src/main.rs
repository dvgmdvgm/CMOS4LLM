use clap::{Parser, Subcommand};

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
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Hello => {
            println!("CMOS v{} — Cognitive Memory Operating System", cmos_core::version());
            println!("Daemon: ready");
        }
    }
}
