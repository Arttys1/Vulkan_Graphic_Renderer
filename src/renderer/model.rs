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
    },
    nalgebra_glm as glm,
};

#[derive(Debug, Clone)]
pub struct Model {
    texture: Texture,
    buffer: VertexBuffer,
}

impl Model {
    pub fn read(renderer: &Renderer, model_url: &str, texture_url: &str) -> Result<Self> {
        let mut vertices : Vec<Vertex> = Vec::default();
        let mut indices : Vec<u32> = Vec::default();
        read_model(model_url, &mut vertices, &mut indices)?;
        
        let buffer = VertexBuffer::allocate_(renderer, vertices, indices)?;
        let texture = Texture::new(renderer, texture_url)?;
        Ok(Model {
            texture,
            buffer,
        })
    }

    pub fn construct(buffer: VertexBuffer, texture: Texture) -> Result<Self> {
        if !(buffer.is_allocated() && texture.is_allocated()) {
            return Err(anyhow!("buffer and texture must be allocated before"));
        }
        Ok(Model {
            texture,
            buffer,
        })
    }

    pub fn clean(&mut self) {
        self.texture.clean();
        self.buffer.clean();
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn buffer(&self) -> &VertexBuffer {
        &self.buffer
    }

    pub fn reload_swapchain(&mut self,
        swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>,) -> Result<()> 
    {
        let a = &mut self.texture; 
        a.reload_swapchain(swapchain_images, descriptor_set_layout, uniform_buffers)?;
        Ok(())
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