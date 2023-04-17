mod app;
mod model;
mod vk_command_buffer;
mod vk_depth_object;
mod vk_descriptor_layout;
mod vk_descriptor_pool;
mod vk_framebuffer;
mod vk_generate_mipmaps;
mod vk_image_view;
mod vk_instance;
mod vk_logical_device;
mod vk_msaa;
mod vk_physical_device;
mod vk_pipeline;
mod vk_render_pass;
mod vk_single_time_cmd;
mod vk_swapchain;
mod vk_sync_object;
mod vk_texture_image;
mod vk_texture_sampler;
mod vk_uniform_buffer;
mod vk_vertex;
mod vk_vertex_buffer;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use std::time::{Duration, Instant};

use vulkanalia::prelude::v1_0::*;

use anyhow::Result;

use app::App;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

const PERFORMANCE_INTERVAL: Duration = Duration::from_secs(1);
struct PerformanceMetrics {
    last_update_time: Instant,
    slowest_draw_time: Duration,
    fastest_draw_time: Duration,
    total_draw_time: Duration,
    total_draw_calls: u32,
}
impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            last_update_time: Instant::now(),
            slowest_draw_time: Duration::from_secs(0),
            fastest_draw_time: Duration::from_secs(30),
            total_draw_time: Duration::from_secs(0),
            total_draw_calls: 0,
        }
    }
}
impl PerformanceMetrics {
    fn update(&mut self, draw_time: Duration) {
        self.total_draw_calls += 1;
        self.total_draw_time += draw_time;
        if draw_time > self.slowest_draw_time {
            self.slowest_draw_time = draw_time;
        }
        if draw_time < self.fastest_draw_time {
            self.fastest_draw_time = draw_time;
        }
        if self.last_update_time.elapsed() > PERFORMANCE_INTERVAL {
            self.report();
            *self = Self::default();
        }
    }

    fn report(&self) {
        log::info!(
            "Slowest {:?} Fastest {:?} Average {:?} Total draw {}",
            self.slowest_draw_time,
            self.fastest_draw_time,
            self.total_draw_time / self.total_draw_calls,
            self.total_draw_calls
        );
    }
}

pub fn start(app_name: &str) -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(app_name)
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut performance_metrics = PerformanceMetrics::default();

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !app.destroying && !app.minimized => {
                let instant = Instant::now();
                unsafe { app.render(&window) }.expect("Failed to render");
                performance_metrics.update(instant.elapsed());
            }
            // Destroy our Vulkan app.
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                app.destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe {
                    app.device
                        .device_wait_idle()
                        .expect("Failed to wait for device to idle");
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width == 0 || size.height == 0 {
                    app.minimized = true;
                } else {
                    app.minimized = false;
                    app.resized = true;
                }
            }
            _ => {}
        }
    });
}
