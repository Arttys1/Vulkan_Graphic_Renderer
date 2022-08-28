pub mod triangle;
pub mod mesh;
use std::sync::Arc;

use crate::{
    renderer::{vertex::Vertex, uniformbuffers::MatrixShaderObject},
    tools::texture::Texture
};

pub trait Object {
    fn vertices(&self) -> &[Vertex];
    fn indices (&self) -> &[u32];
    fn texture (&self) -> Option<Arc<Texture>>;
    fn set_fn_update_matrix(&mut self, f: fn(usize, f32, u32, u32) -> MatrixShaderObject);
    fn get_fn_update_matrix(&self) -> Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>;
}