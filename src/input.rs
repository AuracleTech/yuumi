use cgmath::Vector3;
use std::sync::mpsc::Receiver;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};

enum InputEvent {
    MouseMoved { x: f64, y: f64 },
    MousePressed { button: winit::event::MouseButton },
    MouseReleased { button: winit::event::MouseButton },
    KeyPressed { key: winit::event::VirtualKeyCode },
    KeyReleased { key: winit::event::VirtualKeyCode },
}

pub(crate) fn handle_inputs(receiver: Receiver<Event<()>>) {


}
