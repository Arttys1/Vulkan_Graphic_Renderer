#![allow(dead_code)]
mod renderer;
use renderer::renderer::Renderer;
mod tools;
mod object;

use chrono::{DateTime, Local, Duration};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};
use anyhow::Result;
use nalgebra_glm as glm;

use crate::{renderer::{vertex::Vertex, uniformbuffers::MatrixShaderObject}, object::{triangle::Triangle, Object, mesh::Mesh, rectangle::Rectangle, cube::Cube}};
use tools::loader::Loader;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Renderer (Rust)")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)?;

    // App
    let mut app = Renderer::create(&window)?;
    fill_app(&mut app)?;

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
            }
            _ => {}
        }
    });
}

fn fill_app(app: &mut Renderer) -> Result<()> {
    let mut loader = Loader::default();
    let vertices : Vec::<Vertex> = vec![
        Vertex::new(glm::vec3(-0.5, -0.5, 0.0),glm::vec3(1.0, 1.0, 1.0),glm::vec2(1.0, 0.0)),
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
    let f = |model_index: usize, elapsed: f32, width: u32, height: u32| -> MatrixShaderObject {
        let y = (((model_index % 2) as f32) * 2.5) - 1.25;
        let z = (((model_index / 2) as f32) * -2.0) + 1.0;

        let model = glm::translate(
            &glm::identity(),
            &glm::vec3(0.0, y, z),
        );    
        let model = glm::rotate(
            &model,
            elapsed * glm::radians(&glm::vec1(90.0))[0],
            &glm::vec3(0.0, 0.0, 1.0),
        );
        let view = glm::look_at(
            &glm::vec3(6.0f32, 0.0, 2.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 0.0, 1.0),
        );
        let mut proj = glm::perspective_rh_zo(
            width as f32 / height as f32,
            glm::radians(&glm::vec1(45.0))[0],
            0.1,
            10.0,
        );        
        proj[(1, 1)] *= -1.0;

        MatrixShaderObject::construct(view, model, proj)

    };

    const TEXTURE_VIKING: &str = "resources/viking_room.png";
    const TEXTURE_STATUE: &str = "resources/texture.png";
    const MODEL_PATH: &str ="resources/viking_room.obj";
    let texture_statue = loader.load_texture(&TEXTURE_STATUE.to_string())?;
    let texture_viking = loader.load_texture(&TEXTURE_VIKING.to_string())?;
    let mut triangle = Rectangle::new([vertices[0], vertices[1], vertices[2], vertices[3]],  Some(texture_viking.clone()));
    triangle.set_fn_update_matrix(f);
    let model = loader.load_model(&MODEL_PATH.to_string())?;
    let mut viking_room = Mesh::new(model.clone(), Some(texture_viking.clone()));
    let mut statue_room = Mesh::new(model, Some(texture_statue.clone()));
    let mut cube = Cube::from_one(vertices[0], 1.0, 1.0, 1.0, Some(texture_statue.clone()));
    let mut double_face = Mesh::construct(vertices, indices, Some(texture_statue));
    cube.set_fn_update_matrix(f);
    double_face.set_fn_update_matrix(f);
    viking_room.set_fn_update_matrix(f);
    statue_room.set_fn_update_matrix(f);
    app.add_object(&triangle)?;
    app.add_object(&cube)?;
    app.add_object(&viking_room)?;
    app.add_object(&statue_room)?;
    app.add_object(&double_face)?;
    Ok(())
}