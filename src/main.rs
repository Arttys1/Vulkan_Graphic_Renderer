#![allow(dead_code)]
mod renderer;
use renderer::renderer::Renderer;

use chrono::{DateTime, Local, Duration};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};

use anyhow::Result;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // App
    let mut app = Renderer::create(&window)?;
    let mut destroying = false;
    let mut minimized = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // Render a frame if our Vulkan app is not being destroyed.
            Event::MainEventsCleared if !destroying && !minimized => {
                let start: DateTime<Local> = Local::now(); 
                app.render(&window).expect("Failed to render.");
                let since = Local::now().signed_duration_since(start);
                let wait_time = match (Duration::milliseconds(16) - since).to_std() {
                    Ok(d) => d,
                    Err(_) => std::time::Duration::ZERO,
                };
                std::thread::sleep(wait_time); //60fps
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                if size.width == 0 || size.height == 0 {
                    minimized = true;
                } else {
                    minimized = false;
                    app.must_resize();
                }
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Left) if app.models > 1 => app.models -= 1,
                        Some(VirtualKeyCode::Right) if app.models < 4 => app.models += 1,
                        _ => { }
                    }
                }
            }
            // Destroy our Vulkan app.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                app.clean();
            }
            _ => {}
        }
    });
}
