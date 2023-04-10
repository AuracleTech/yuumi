// #![allow(
//     dead_code,
//     unused_variables,
//     clippy::too_many_arguments,
//     clippy::unnecessary_wraps
// )]

mod app;
mod vk_command_buffer;
mod vk_framebuffer;
mod vk_instance;
mod vk_logical_device;
mod vk_physical_device;
mod vk_pipeline;
mod vk_render_pass;
mod vk_swapchain;
mod vk_sync_object;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use vulkanalia::prelude::v1_0::*;

use anyhow::Result;

use app::App;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub fn start() -> Result<()> {
    pretty_env_logger::init();

    // Window

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !app.destroying => unsafe { app.render(&window) }.unwrap(),
            // Destroy our Vulkan app.
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                app.destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe {
                    app.device.device_wait_idle().unwrap();
                }
                unsafe {
                    app.destroy();
                }
            }
            _ => {}
        }
    });
}
