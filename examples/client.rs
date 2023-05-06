use std::sync::{Arc, Mutex};

use anyhow::Result;
use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};
use yuumi::{App, CameraController, CameraProjectionKind};

fn main() -> Result<()> {
    // Window
    let window_title = "Yuumi";
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title(window_title)
        .with_inner_size(winit::dpi::LogicalSize::new(1600, 900))
        .build(&event_loop)?;

    // Center window
    let window_size = window.outer_size();
    let monitor_size = window
        .current_monitor()
        .expect("Failed to get monitor")
        .size();
    let x = (monitor_size.width - window_size.width) / 2;
    let y = (monitor_size.height - window_size.height) / 2;
    window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));

    // Event Loop
    window.set_cursor_grab(winit::window::CursorGrabMode::Confined)?;
    window.set_cursor_visible(false);

    // Set the initial cursor position to the center of the window
    let cursor_position = winit::dpi::PhysicalPosition::new(
        window.inner_size().width as f64 / 2.0,
        window.inner_size().height as f64 / 2.0,
    );

    // App
    let app = Arc::new(Mutex::new(App::new_windowed(&window)?));

    // Assets
    let mut camera_controller = CameraController {
        aim_sensitivity: 0.03,
        speed_factor: 4,
        speed: 1.00,
        yaw: 200.0,
        pitch: -20.0,
        mouse_pos_last_x: 0.0,
        mouse_pos_last_y: 0.0,
        min_fov_y: 10.0,
        max_fov_y: 130.0,
    };

    let (sender, receiver) = std::sync::mpsc::channel::<Event<()>>();

    // Thread input
    let app_arc_thread_input = Arc::clone(&app);
    let asset1 = Arc::clone(&app_arc_thread_input.lock().unwrap().assets);
    std::thread::spawn(move || loop {
        let event = receiver.recv().unwrap();
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::W) => {
                                let mut assets = asset1.write().expect("Failed to lock assets");
                                let camera = assets
                                    .cameras
                                    .get_mut("main")
                                    .expect("Failed to get camera");
                                let forward = camera.quat * Vector3::unit_z();
                                camera.pos -= forward * camera_controller.speed;
                                camera.update();
                            }
                            Some(VirtualKeyCode::S) => {
                                let mut assets = asset1.write().expect("Failed to lock assets");
                                let camera = assets
                                    .cameras
                                    .get_mut("main")
                                    .expect("Failed to get camera");
                                let forward = camera.quat * Vector3::unit_z();
                                camera.pos += forward * camera_controller.speed;
                                camera.update();
                            }
                            Some(VirtualKeyCode::A) => {
                                let mut assets = asset1.write().expect("Failed to lock assets");
                                let camera = assets
                                    .cameras
                                    .get_mut("main")
                                    .expect("Failed to get camera");
                                let right = camera.quat * Vector3::unit_x();
                                camera.pos -= right * camera_controller.speed;
                                camera.update();
                            }
                            Some(VirtualKeyCode::D) => {
                                let mut assets = asset1.write().expect("Failed to lock assets");
                                let camera = assets
                                    .cameras
                                    .get_mut("main")
                                    .expect("Failed to get camera");
                                let right = camera.quat * Vector3::unit_x();
                                camera.pos += right * camera_controller.speed;
                                camera.update();
                            }
                            Some(VirtualKeyCode::Space) => {
                                let mut assets = asset1.write().expect("Failed to lock assets");
                                let camera = assets
                                    .cameras
                                    .get_mut("main")
                                    .expect("Failed to get camera");
                                let up = camera.quat * Vector3::unit_y();
                                camera.pos += up * camera_controller.speed;
                                camera.update();
                            }
                            Some(VirtualKeyCode::LControl) => {
                                let mut assets = asset1.write().expect("Failed to lock assets");
                                let camera = assets
                                    .cameras
                                    .get_mut("main")
                                    .expect("Failed to get camera");
                                let up = camera.quat * Vector3::unit_y();
                                camera.pos -= up * camera_controller.speed;
                                camera.update();
                            }
                            Some(VirtualKeyCode::LShift) => {
                                camera_controller.speed_factor =
                                    match camera_controller.speed_factor {
                                        4 => 8,
                                        8 => 24,
                                        24 => 4,
                                        _ => 4,
                                    };
                            }
                            Some(VirtualKeyCode::Escape) => {
                                let app =
                                    &mut app_arc_thread_input.lock().expect("Failed to lock app");
                                app.destroy();
                            }
                            _ => {}
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });

    event_loop.run(move |event, _, control_flow| {
        let app = &mut app.lock().unwrap();
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared if app.running && !app.minimized => {
                unsafe { app.render(&window) }.expect("Failed to render");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
                app.destroy();
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
                // Update camera aspect ratio
                let window_inner_size = window.inner_size();
                let mut assets = app.assets.write().expect("Failed to lock assets");
                let camera = assets
                    .cameras
                    .get_mut("main")
                    .expect("Failed to get camera");
                camera.set_aspect_ratio(
                    window_inner_size.width as f32 / window_inner_size.height as f32,
                );
            }

            // mouse event
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                // Set the new cursor position to the center of the window
                window
                    .set_cursor_position(PhysicalPosition::new(
                        window.inner_size().width as f64 / 2.0,
                        window.inner_size().height as f64 / 2.0,
                    ))
                    .expect("Failed to set cursor position");

                let mut assets = app.assets.write().expect("Failed to lock assets");
                let camera = assets
                    .cameras
                    .get_mut("main")
                    .expect("Failed to get camera");

                let mouse_x_delta = position.x - cursor_position.x;
                let mouse_y_delta = position.y - cursor_position.y;
                camera_controller.mouse_pos_last_x = position.x;
                camera_controller.mouse_pos_last_y = position.y;

                camera_controller.yaw -= mouse_x_delta as f32 * camera_controller.aim_sensitivity;
                camera_controller.pitch -= mouse_y_delta as f32 * camera_controller.aim_sensitivity;

                camera_controller.pitch = camera_controller.pitch.clamp(-89.9, 89.9);
                camera_controller.yaw = camera_controller.yaw.rem_euclid(360.0);

                let quat_yaw =
                    Quaternion::from_axis_angle(Vector3::unit_y(), Deg(camera_controller.yaw));
                let quat_pitch =
                    Quaternion::from_axis_angle(Vector3::unit_x(), Deg(camera_controller.pitch));
                camera.quat = quat_yaw * quat_pitch;
                camera.update(); // OPTIMIZE stack up all updates and update only once at end of update loop, maybe use boolean or sum
            }

            // Mouse scroll change FOV
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, phase, .. },
                ..
            } => {
                if phase == winit::event::TouchPhase::Moved {
                    let mut assets = app.assets.write().expect("Failed to lock assets");
                    let camera = assets
                        .cameras
                        .get_mut("main")
                        .expect("Failed to get camera");
                    match camera.projection_kind {
                        CameraProjectionKind::Perspective {
                            aspect_ratio,
                            far,
                            fov_y,
                            near,
                        } => {
                            match delta {
                                winit::event::MouseScrollDelta::LineDelta(_delta_x, delta_y) => {
                                    camera.projection_kind = CameraProjectionKind::Perspective {
                                        aspect_ratio,
                                        fov_y: (fov_y + delta_y).clamp(
                                            camera_controller.min_fov_y,
                                            camera_controller.max_fov_y,
                                        ),
                                        near,
                                        far,
                                    };
                                    camera.update();
                                }
                                // TODO support whatever the hell this is
                                winit::event::MouseScrollDelta::PixelDelta(_) => unimplemented!(),
                            }
                        }
                        _ => {}
                    };
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input,
                        device_id,
                        is_synthetic,
                    },
                ..
            } => {
                sender
                    .send(Event::WindowEvent {
                        window_id: window.id(),
                        event: WindowEvent::KeyboardInput {
                            input,
                            device_id,
                            is_synthetic,
                        },
                    })
                    .expect("Failed to send event to input thread");

                match input.virtual_keycode {
                    Some(VirtualKeyCode::Escape) => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
