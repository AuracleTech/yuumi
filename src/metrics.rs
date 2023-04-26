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
    pub(crate) fn start(&mut self) {
        self.start = Instant::now();
    }

    pub(crate) fn start_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    pub(crate) fn end_frame(&mut self) {
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

// pub(crate) fn print_memory_usage() {
//     use memory_stats::memory_stats;
//     if let Some(usage) = memory_stats() {
//         log::info!("Virtual memory usage: {}", format_size(usage.virtual_mem));
//         log::info!("Physical memory usage: {}", format_size(usage.physical_mem));
//     }
// }

// fn format_size(mut size: usize) -> String {
//     let units = ["B", "KB", "MB", "GB", "TB", "PB"];
//     let mut index = 0;

//     while size >= 1024 && index < units.len() - 1 {
//         size /= 1024;
//         index += 1;
//     }

//     format!("{} {}", size, units[index])
// }
