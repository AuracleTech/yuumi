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

pub fn start(app_name: &str) -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(app_name)
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut last_performance_update = Instant::now();

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !app.destroying && !app.minimized => {
                let instant = Instant::now();

                unsafe { app.render(&window) }.expect("Failed to render");

                let elapsed = instant.elapsed();

                if Instant::now() - last_performance_update > Duration::from_millis(200) {
                    log::info!("render {:?}", elapsed);
                    last_performance_update = Instant::now();
                }
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
