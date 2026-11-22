//! Core utilities with no internal dependencies.
//!
//! This crate represents Layer 0 - shared utilities used across all other crates.

use std::time::{Duration, Instant};

/// Timing utility for performance measurements.
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn log_elapsed(&self) {
        tracing::info!("{}: {:?}", self.label, self.elapsed());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.log_elapsed();
    }
}
