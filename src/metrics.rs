//! This module provides a way to gather metrics for the NES
//!

use std::time::Duration;
use std::time::Instant;

use log::debug;

#[derive(Debug)]
struct RawMetrics {
    record_start: Instant,
    frames_rendered: usize,
}

#[derive(Debug)]
pub struct Metrics {
    pub recorded_time: Duration,
    pub frames_per_second: usize,
}

pub struct Collector {
    collecting: RawMetrics,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            collecting: RawMetrics {
                record_start: Instant::now(),
                frames_rendered: 0,
            },
        }
    }

    pub fn collect(&mut self) -> Metrics {
        debug!("Raw metrics: {:?}", self.collecting);
        let recorded_time = Instant::now() - self.collecting.record_start;
        let frames_per_second =
            (self.collecting.frames_rendered as u128) * 1_000_000 / recorded_time.as_micros();

        let metrics = Metrics {
            recorded_time,
            frames_per_second: frames_per_second as usize,
        };

        self.collecting.record_start = Instant::now();
        self.collecting.frames_rendered = 0;

        metrics
    }

    pub fn observe_frame_ready(&mut self) {
        self.collecting.frames_rendered += 1;
    }
}
