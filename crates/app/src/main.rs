use std::path::PathBuf;

use clap::{Parser, Subcommand};
use music_visualiser_core::{AudioEngine, AudioMode, MappingMatrix, RenderGraph, Scheduler};
use tracing_subscriber::EnvFilter;

fn main() -> music_visualiser_core::Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Commands::Live { preset } => run_live(preset.as_deref()),
        Commands::Precompute { input, output } => run_precompute(&input, &output),
    }
}

fn run_live(preset: Option<&str>) -> music_visualiser_core::Result<()> {
    tracing::info!(preset, "starting live mode");

    let audio = AudioEngine::new(AudioMode::Live);
    let analysis = audio.start()?;
    let mut scheduler = Scheduler::new();
    let mut mappings = MappingMatrix::new();
    let mut render = RenderGraph::new();

    let frame = analysis.sample_at(0.0);
    let _ = scheduler.tick(0.0, &frame);
    let updates = mappings.evaluate(&frame);
    render.apply_updates(updates);
    render.draw()
}

fn run_precompute(input: &PathBuf, output: &PathBuf) -> music_visualiser_core::Result<()> {
    tracing::info!(?input, ?output, "running precompute pipeline");
    // The actual pipeline will decode `input`, run offline analysis, and emit a
    // cache file at `output`. For now we simply acknowledge the intent.
    std::fs::write(output, b"{}")?;
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Feature-rich music visualiser", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the application in low-latency live mode.
    Live {
        /// Optional preset file to load on startup.
        #[arg(short, long)]
        preset: Option<String>,
    },
    /// Analyse an audio file ahead of time and persist the results.
    Precompute {
        /// Path to the audio file that should be analysed.
        input: PathBuf,
        /// Output path for the generated analysis cache.
        output: PathBuf,
    },
}
