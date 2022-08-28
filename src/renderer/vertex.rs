use std::{
    mem::size_of,
    hash::{Hash, Hasher},
};
use vulkanalia::{
    prelude::v1_0::*
};
use nalgebra_glm as glm;

//================================================
// Vertex
//================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pos: glm::Vec3,
    color: glm::Vec3,
    tex_coord: glm::Vec2,
}

impl Vertex {
    pub fn new(pos: glm::Vec3, color: glm::Vec3, tex_coord: glm::Vec2) -> Self {
        Self { pos, color, tex_coord }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<glm::Vec3>() as u32)
            .build();
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<glm::Vec3>() + size_of::<glm::Vec3>()) as u32)
            .build();
        [pos, color, tex_coord]
    }

    pub fn pos(&self) -> glm::Vec3 { self.pos }
    pub fn color(&self) -> glm::Vec3 { self.color }
    pub fn tex_coord(&self) -> glm::Vec2 { self.tex_coord }
    pub fn set_tex_coord(&mut self, tex_coord: glm::Vec2) { self.tex_coord = tex_coord; }
    pub fn set_color(&mut self, color: glm::Vec3) { self.color = color; }
    pub fn set_pos(&mut self, pos: glm::Vec3) { self.pos = pos; }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos &&
        self.color == other.color &&
        self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}