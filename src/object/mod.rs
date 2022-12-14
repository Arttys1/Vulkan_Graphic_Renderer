pub mod mesh;
pub mod triangle;
pub mod rectangle;
pub mod cube;
pub mod circle;
pub mod sphere;
use std::{sync::Arc, collections::HashMap};

use crate::{
    renderer::{vertex::Vertex, uniformbuffers::MatrixShaderObject},
    tools::texture::Texture
};

pub trait Object {
    fn vertices(&self) -> &[Vertex];
    fn indices (&self) -> &[u32];
    fn texture (&self) -> Option<Arc<Texture>>;
    fn set_texture(&mut self, texture: Arc<Texture>);
    fn set_fn_update_matrix(&mut self, f: fn(usize, f32, u32, u32) -> MatrixShaderObject);
    fn get_fn_update_matrix(&self) -> Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>;
}

pub(crate) fn add_unique_vertex(hashmap: &mut HashMap<Vertex, u32>, 
    vertices: &mut Vec<Vertex>,
    indices : &mut Vec<u32>,
    vertex :Vertex) {
        if !hashmap.contains_key(&vertex) {
            hashmap.insert(vertex, vertices.len() as u32);
			vertices.push(vertex);
		}

		indices.push(hashmap[&vertex]);
    }