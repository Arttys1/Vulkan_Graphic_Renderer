use std::sync::Arc;

use super::renderer::Renderer;

use {
    anyhow::{Result, anyhow},
    std::{
        fs::File,
        io::BufReader,
        collections::HashMap,
    },
    vulkanalia::prelude::v1_0::*,
    super::{
        texture::Texture,
        vertexbuffers::VertexBuffer,
        vertex::Vertex, 
        uniformbuffers::UniformBuffer,
        descriptor::Descriptor,
    },
    nalgebra_glm as glm,
};

#[derive(Debug, Clone)]
pub struct Model {
    texture: Texture,
    buffer: VertexBuffer,
    uniform_buffer: UniformBuffer,
    descriptor: Descriptor,
}

impl Model {
    pub fn read(device: Arc<Device>, instance: &Instance, 
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout, model_path: &str, 
        texture_path: &str) -> Result<Self> 
    {
        let mut vertices : Vec<Vertex> = Vec::default();
        let mut indices : Vec<u32> = Vec::default();
        read_model(model_path, &mut vertices, &mut indices)?;
        
        let buffer = VertexBuffer::new(device.clone(), instance, physical_device, command_pool, graphics_queue, vertices, indices)?;
        let texture = Texture::new(device.clone(), instance,physical_device,command_pool,graphics_queue, texture_path)?;
        let mut uniform_buffer = UniformBuffer::new(device.clone(), instance,physical_device,swapchain_images)?;
        uniform_buffer.set_fn_update_ubo(Renderer::update_ubo);
        uniform_buffer.set_fn_update_push_constant(Renderer::update_push_constant);
        let descriptor = Descriptor::new(device.clone(),             
            swapchain_images,
            descriptor_set_layout, 
            &uniform_buffer, 
            &texture)?;
        Ok(Model {
            texture,
            buffer,
            uniform_buffer,
            descriptor,
        })
    }

    pub fn construct(buffer: VertexBuffer, texture: Texture, uniform_buffer: UniformBuffer, descriptor: Descriptor)
-> Result<Self> {
        if !(buffer.is_allocated() && texture.is_allocated() && uniform_buffer.is_allocated() && descriptor.is_allocated()) {
            return Err(anyhow!("vertex_buffer, texture and uniform_buffer must be allocated before"));
        }
        Ok(Model {
            texture,
            buffer,
            uniform_buffer,
            descriptor,
        })
    }

    pub fn clean(&mut self) {
        self.texture.clean();
        self.buffer.clean();
        self.uniform_buffer.clean();
        self.descriptor.clean();
    }

    pub fn reload_swapchain(&mut self,
        instance: &Instance, 
        physical_device: vk::PhysicalDevice,
        swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout) -> Result<()> 
    {
        self.uniform_buffer.reload_swapchain_models(instance, physical_device, swapchain_images)?;
        self.descriptor.reload_swapchain(swapchain_images, descriptor_set_layout, &self.uniform_buffer, &self.texture)?;
        Ok(())
    }

    pub fn update(&mut self, device: &Device, swapchain_extent: vk::Extent2D, image_index: usize) -> Result<()> {
        unsafe {
            self.uniform_buffer.update_uniform_buffer(device, swapchain_extent, image_index)?;
            Ok(())
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn buffer(&self) -> &VertexBuffer {
        &self.buffer
    }

    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    pub fn uniform_buffer(&self) -> &UniformBuffer {
        &self.uniform_buffer
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.clean();
    }
}

//================================================
// load Model
//================================================

pub fn read_model(url: &str, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) -> Result<()> {
    let mut reader = BufReader::new(File::open(url)?);

    let (models, _) = tobj::load_obj_buf(&mut reader, true, |_| {
        Ok((vec![tobj::Material::empty()], HashMap::new()))
    })?;

    let mut unique_vertices = HashMap::new();

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex::new (
                glm::vec3 (
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                glm::vec3(1.0, 1.0, 1.0),
                glm::vec2 (
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),   
            );

            if let Some(index) = unique_vertices.get(&vertex) {
                indices.push(*index as u32);
            } else {
                let index = vertices.len();
                unique_vertices.insert(vertex, index);
                vertices.push(vertex);
                indices.push(index as u32);
            }
        }
    }    

    Ok(())
}