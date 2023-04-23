use std::time::{Duration, Instant};

const CYCLE_REPORT_INTERVAL: Duration = Duration::from_secs(1);
#[derive(Debug)]
pub(crate) struct Cycle {
    start: Instant,
    frame_start: Instant,
    slowest_render: Duration,
    fastest_render: Duration,
    total_render: Duration,
    total_frames: u32,
}
impl Default for Cycle {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            frame_start: Instant::now(),
            slowest_render: Duration::from_secs(0),
            fastest_render: Duration::from_secs(30),
            total_render: Duration::from_secs(0),
            total_frames: 0,
        }
    }
}
impl Cycle {
    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    pub fn end_frame(&mut self) {
        self.total_frames += 1;
        let elapsed_time = self.frame_start.elapsed();
        self.total_render += elapsed_time;

        if elapsed_time > self.slowest_render {
            self.slowest_render = elapsed_time;
        }
        if elapsed_time < self.fastest_render {
            self.fastest_render = elapsed_time;
        }

        if self.start.elapsed() > CYCLE_REPORT_INTERVAL {
            log::info!(
                "Slowest {:?} Fastest {:?} Average {:?} Draw calls {}",
                self.slowest_render,
                self.fastest_render,
                self.total_render / self.total_frames,
                self.total_frames
            );
            *self = Self::default();
        }
    }
}

#[derive(Debug)]
pub(crate) struct Metrics {
    pub(crate) engine_start: Instant,
    pub(crate) cycle: Cycle,
    pub(crate) total_frames: u64,
}
impl Default for Metrics {
    fn default() -> Self {
        Self {
            engine_start: Instant::now(),
            cycle: Cycle::default(),
            total_frames: 0,
        }
    }
}
