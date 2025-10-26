//! Core library for the Music Visualiser application.
//!
//! The goal of this crate is to provide a lightweight yet well-structured
//! foundation that mirrors the long term architecture of the project. The
//! individual modules intentionally expose simple data structures and behaviour
//! so that higher level features can rely on stable APIs while the heavy lifting
//! (real-time audio capture, GPU rendering, complex scheduling) is implemented
//! incrementally.

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
pub use assets::{AssetHandle, AssetStore, StlAsset};
pub use audio::{AnalysisHandle, AudioEngine, AudioMode};
pub use config::{AppConfig, AudioConfig};
pub use error::{MusicVizError, Result};
pub use mapping::{MappingDescriptor, MappingMatrix, ParameterUpdate};
pub use record::{Recorder, RecordingSettings};
pub use render::RenderGraph;
pub use scene::{SceneDescriptor, SceneInstance, SceneKind, StlMode};
pub use timeline::{PlaybackClock, ScheduledEvent, Scheduler};
