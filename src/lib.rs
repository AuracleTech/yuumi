mod app;
mod vertex;
mod vk_command_buffer;
mod vk_descriptor_layout;
mod vk_descriptor_pool;
mod vk_framebuffer;
mod vk_instance;
mod vk_logical_device;
mod vk_physical_device;
mod vk_pipeline;
mod vk_render_pass;
mod vk_swapchain;
mod vk_sync_object;
mod vk_uniform_buffer;
mod vk_vertex_buffer;

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
            Event::MainEventsCleared if !app.destroying && !app.minimized => {
                unsafe { app.render(&window) }.unwrap()
            }
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
