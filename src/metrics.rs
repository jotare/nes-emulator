//! This module provides a way to gather metrics for the NES
//!

use std::time::Duration;
use std::time::Instant;

use log::debug;

#[derive(Debug)]
struct RawMetrics {
    record_start: Instant,
    clocks: u64,
    frames_rendered: usize,
}

#[derive(Debug)]
pub struct Metrics {
    pub recorded_time: Duration,
    pub clock_speed_mhz: usize,
    pub frames_per_second: usize,
}

pub struct Collector {
    collecting: RawMetrics,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            collecting: RawMetrics::default(),
        }
    }

    pub fn collect(&mut self) -> Metrics {
        debug!("Raw metrics: {:?}", self.collecting);
        let recorded_time = Instant::now() - self.collecting.record_start;
        let clock_speed_mhz =
            (self.collecting.clocks as u128) * 1_000_000 / recorded_time.as_micros() / 1_000_000;
        let frames_per_second =
            (self.collecting.frames_rendered as u128) * 1_000_000 / recorded_time.as_micros();

        let metrics = Metrics {
            recorded_time,
            clock_speed_mhz: clock_speed_mhz as usize,
            frames_per_second: frames_per_second as usize,
        };
        debug!("Metrics: {:?}", metrics);

        self.collecting.reset();

        metrics
    }

    pub fn observe_system_clocks(&mut self, clocks: u64) {
        self.collecting.clocks += clocks;
    }

    pub fn observe_frame_ready(&mut self) {
        self.collecting.frames_rendered += 1;
    }
}

impl RawMetrics {
    fn reset(&mut self) {
        self.record_start = Instant::now();
        self.clocks = 0;
        self.frames_rendered = 0;
    }
}

impl Default for RawMetrics {
    fn default() -> Self {
        Self {
            record_start: Instant::now(),
            clocks: 0,
            frames_rendered: 0,
        }
    }
}
