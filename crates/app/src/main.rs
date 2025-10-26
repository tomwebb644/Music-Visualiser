use std::f32::consts::TAU;

use clap::{Args, Parser, Subcommand};
use music_visualiser_core::{
    AppConfig, AudioEngine, AudioMode, MappingMatrix, PlaybackClock, Recorder, RecordingSettings,
    RenderGraph, SceneDescriptor, SceneInstance, ScheduledEvent, Scheduler,
};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> music_visualiser_core::Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Live(options) => run_live(options),
        Command::Precompute(options) => run_precompute(options),
    }
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn run_live(options: LiveOptions) -> music_visualiser_core::Result<()> {
    let config = AppConfig::live_defaults();
    let mut audio = AudioEngine::with_sample_rate(AudioMode::Live, config.audio.sample_rate);
    let mut mapping = MappingMatrix::new();
    let mut render_graph = RenderGraph::new();
    let descriptor = SceneDescriptor::live_demo();
    let mut scene = SceneInstance::new(descriptor);
    let mut scheduler = Scheduler::new();
    scheduler.set_events(vec![
        ScheduledEvent::new(2.0, "section-a"),
        ScheduledEvent::new(4.0, "section-b"),
    ]);
    let mut clock = PlaybackClock::default();
    let mut recorder = Recorder::new(RecordingSettings::default());

    for block_index in 0..options.blocks {
        let samples = synthesise_block(block_index, options.block_size, config.audio.sample_rate);
        let frame = audio.process_live_block(&samples)?;
        mapping.apply_from_frame(&frame);
        render_graph.apply_updates(mapping.updates());
        scene.apply_updates(render_graph.last_updates());
        clock.advance(options.block_size as f32 / config.audio.sample_rate as f32);
        scheduler.tick(&clock, &mut scene, &frame);
        recorder.record_frame(&frame);

        info!(
            block = block_index,
            rms = frame.rms,
            centroid = frame.spectral_centroid,
            beat = frame.beat_confidence,
            "processed live block"
        );
        info!(
            intensity = scene.intensity,
            motion = scene.motion,
            beat = scene.beat_emphasis,
            "scene updated"
        );
    }

    info!(frames = recorder.recorded_frames(), "live session complete");
    Ok(())
}

fn run_precompute(options: PrecomputeOptions) -> music_visualiser_core::Result<()> {
    warn!(?options.input, "precomputed mode is not yet available");
    Ok(())
}

fn synthesise_block(block_index: u32, block_size: usize, sample_rate: u32) -> Vec<f32> {
    let base_frequency = 110.0;
    let sweep = 15.0 * block_index as f32;
    let amplitude = 0.5 + 0.5 * ((block_index as f32) * 0.2).sin();
    let mut output = Vec::with_capacity(block_size);

    for sample_index in 0..block_size {
        let absolute_index = block_index as usize * block_size + sample_index;
        let time = absolute_index as f32 / sample_rate as f32;
        let frequency = base_frequency + sweep;
        let value = (time * frequency * TAU).sin() * amplitude;
        output.push(value);
    }

    output
}

#[derive(Parser)]
#[command(author, version, about = "Music Visualiser command line interface")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the application in live mode using a synthetic audio source.
    Live(LiveOptions),
    /// Stub for the deterministic precomputed mode pipeline.
    Precompute(PrecomputeOptions),
}

#[derive(Args)]
struct LiveOptions {
    /// Number of synthetic blocks to process.
    #[arg(long, default_value_t = 8)]
    blocks: u32,
    /// Number of samples in each block.
    #[arg(long, default_value_t = 1024)]
    block_size: usize,
}

#[derive(Args, Debug)]
struct PrecomputeOptions {
    /// Optional path to the audio file that would be analysed.
    #[arg(value_name = "INPUT", default_value = "")]
    input: String,
}
