#![allow(dead_code)]
mod renderer;
use renderer::{renderer::Renderer};

use chrono::{DateTime, Local, Duration};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};
use anyhow::Result;
use nalgebra_glm as glm;

use crate::renderer::{vertex::Vertex, vertexbuffers::*, texture::*, model::*};

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
    {                        
        let vertices : Vec::<Vertex> = vec![
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0),glm::vec3(1.0, 0.0, 0.0),glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, 0.0), glm::vec3(0.0, 1.0, 0.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, 0.0), glm::vec3(0.0, 0.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
            //
            Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(1.0, 0.0, 0.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, -0.5), glm::vec3(0.0, 1.0, 0.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, -0.5), glm::vec3(0.0, 0.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, -0.5), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        
        let indices: Vec<u32> = vec!(
            0, 1, 2, 2, 3, 0,
            //
            4, 5, 6, 6, 7, 4
        );

        let buffer = VertexBuffer::allocate_(&app, vertices, indices)?;
        let texture = Texture::new(&app, "resources/texture.png")?;
        app.add_model(Model::construct(buffer, texture)?);

        const TEXTURE_PATH: &str = "resources/viking_room.png";
        const MODEL_PATH: &str ="resources/viking_room.obj";
        println!("load models...");
        app.add_model(Model::read(&app,MODEL_PATH, TEXTURE_PATH)?);
        app.add_model(Model::read(&app,MODEL_PATH, "resources/texture.png")?);
        app.add_model(Model::read(&app,MODEL_PATH, "resources/texture.png")?);
        println!("models loaded.");
    }
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
