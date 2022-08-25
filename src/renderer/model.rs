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
        renderer::Renderer,
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
    pub fn read(renderer: &Renderer, model_url: &str, texture_url: &str) -> Result<Self> {
        let mut vertices : Vec<Vertex> = Vec::default();
        let mut indices : Vec<u32> = Vec::default();
        read_model(model_url, &mut vertices, &mut indices)?;
        
        let buffer = VertexBuffer::allocate_(renderer, vertices, indices)?;
        let texture = Texture::new(renderer, texture_url)?;
        let uniform_buffer = UniformBuffer::new(renderer)?;
        let data = renderer.get_appdata();
        let device = renderer.get_device();
        let descriptor = Descriptor::new(device, 
            data.swapchain_images(),
            data.descriptor_set_layout(), 
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

    pub fn uniform_buffer_mut(&mut self) -> &UniformBuffer {
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