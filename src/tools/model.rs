use std::{io::BufReader, fs::File, collections::HashMap};

use crate::renderer::vertex::Vertex;
use nalgebra_glm as glm;
use anyhow::{Result, anyhow};
pub struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<u32>, 
}

impl Model {
    pub fn construct(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn vertices(&self) -> &Vec<Vertex> { self.vertices.as_ref() } 
    pub fn indices(&self) -> &Vec<u32> { self.indices.as_ref() }
}


//================================================
// load Model
//================================================

pub(crate) fn load_model(url: &String, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) -> Result<()> {
    let mut reader = BufReader::new(File::open(url)?);

    let (models, _) = match tobj::load_obj_buf(&mut reader, true, |_| {
        Ok((vec![tobj::Material::empty()], HashMap::new()))
    }) {
        Ok(tuple) => tuple,
        Err(e) => return Err(anyhow!(e))
    };

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