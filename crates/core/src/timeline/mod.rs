use std::time::{Duration, Instant};

use crate::analysis::AnalysisFrame;

/// Deterministic playback clock shared between systems.
#[derive(Debug, Clone)]
pub struct PlaybackClock {
    origin: Instant,
    paused_at: Option<Instant>,
}

impl PlaybackClock {
    pub fn start() -> Self {
        Self {
            origin: Instant::now(),
            paused_at: None,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self.paused_at {
            Some(instant) => instant.duration_since(self.origin),
            None => Instant::now().duration_since(self.origin),
        }
    }

    pub fn pause(&mut self) {
        if self.paused_at.is_none() {
            self.paused_at = Some(Instant::now());
        }
    }

    pub fn resume(&mut self) {
        if let Some(paused) = self.paused_at.take() {
            let pause_duration = Instant::now() - paused;
            self.origin += pause_duration;
        }
    }
}

/// Schedules scene transitions and other time-aligned events.
#[derive(Debug, Default)]
pub struct Scheduler {
    events: Vec<ScheduledEvent>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event(&mut self, event: ScheduledEvent) {
        self.events.push(event);
        self.events
            .sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
    }

    pub fn tick(&mut self, now: f32, analysis: &AnalysisFrame) -> Vec<ScheduledEvent> {
        let mut triggered = Vec::new();
        while let Some(event) = self.events.first() {
            if event.timestamp <= now {
                triggered.push(self.events.remove(0));
            } else {
                break;
            }
        }

        for event in &mut triggered {
            event.last_analysis = Some(analysis.clone());
        }

        triggered
    }
}

/// Representation of a future action.
#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    pub timestamp: f32,
    pub label: String,
    pub last_analysis: Option<AnalysisFrame>,
}

impl ScheduledEvent {
    pub fn new<T: Into<String>>(timestamp: f32, label: T) -> Self {
        Self {
            timestamp,
            label: label.into(),
            last_analysis: None,
        }
    }
}
