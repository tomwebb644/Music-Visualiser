//! Core library for the Music Visualiser application.
//!
//! The crate exposes lightweight scaffolding that mirrors the high-level
//! architecture outlined in the project specification. Each module owns a
//! distinct subsystem (audio capture, analysis, scheduling, rendering, etc.)
//! and provides minimal placeholder implementations so that higher level
//! components can be built incrementally without fighting the compiler.

pub mod analysis;
pub mod assets;
pub mod audio;
pub mod config;
pub mod error;
pub mod mapping;
pub mod record;
pub mod render;
pub mod scene;
pub mod timeline;

pub use analysis::{AnalysisEngine, AnalysisFrame, AnalysisSummary};
pub use assets::{AssetStore, StlAsset};
pub use audio::{AnalysisHandle, AudioEngine, AudioMode};
pub use config::{AppConfig, AudioConfig};
pub use error::{MusicVizError, Result};
pub use mapping::{MappingDescriptor, MappingMatrix, ParameterUpdate};
pub use record::{Recorder, RecordingSettings};
pub use render::RenderGraph;
pub use scene::{SceneDescriptor, SceneInstance, StlMode};
pub use timeline::{PlaybackClock, ScheduledEvent, Scheduler};
