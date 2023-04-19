use std::time::{Duration, Instant};

const CYCLE_REPORT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub(crate) struct Metrics {
    pub(crate) engine_start: Instant,
    pub(crate) last_report: Instant,
    pub(crate) total_frames: u64,

    // TODO - Move this to a cycle struct
    pub(crate) cycle_frame_start: Instant,
    pub(crate) cycle_slowest_render: Duration,
    pub(crate) cycle_fastest_render: Duration,
    pub(crate) cycle_total_rendered: Duration,
    pub(crate) cycle_total_render_count: u32,
}
impl Default for Metrics {
    fn default() -> Self {
        Self {
            engine_start: Instant::now(),
            last_report: Instant::now(),
            total_frames: 0,

            cycle_frame_start: Instant::now(),
            cycle_slowest_render: Duration::from_secs(0),
            cycle_fastest_render: Duration::from_secs(30),
            cycle_total_rendered: Duration::from_secs(0),
            cycle_total_render_count: 0,
        }
    }
}
impl Metrics {
    pub(crate) fn start_frame(&mut self) {
        self.cycle_frame_start = Instant::now();
    }
    pub(crate) fn end_frame(&mut self) {
        let elapsed_time = self.cycle_frame_start.elapsed();
        self.total_frames += 1;
        self.cycle_total_render_count += 1;
        self.cycle_total_rendered += elapsed_time;

        if elapsed_time > self.cycle_slowest_render {
            self.cycle_slowest_render = elapsed_time;
        }
        if elapsed_time < self.cycle_fastest_render {
            self.cycle_fastest_render = elapsed_time;
        }

        if self.last_report.elapsed() > CYCLE_REPORT_INTERVAL {
            log::info!(
                "Slowest {:?} Fastest {:?} Average {:?}",
                self.cycle_slowest_render,
                self.cycle_fastest_render,
                self.cycle_total_rendered / self.cycle_total_render_count,
            );
            self.last_report = Instant::now();
            self.cycle_slowest_render = Duration::from_secs(0);
            self.cycle_fastest_render = Duration::from_secs(30);
            self.cycle_total_rendered = Duration::from_secs(0);
            self.cycle_total_render_count = 0;
        }
    }
}
