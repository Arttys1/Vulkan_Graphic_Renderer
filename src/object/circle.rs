use {
    std::{
        sync::Arc,
        collections::HashMap,
    },
    crate::{
        tools::texture::Texture,
        renderer::{
            vertex::Vertex, uniformbuffers::MatrixShaderObject
        }
    },
    super::{ Object, add_unique_vertex},
    nalgebra_glm as glm,
};

pub struct Circle {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,    
    texture: Option<Arc<Texture>>,
    fn_update_matrix : Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>,
}

impl Circle {
    pub fn new(center: Vertex, radius: f32, edge: u8, texture: Option<Arc<Texture>>) -> Self {
        let vec3 = |x: f32, y: f32, z: f32| -> glm::Vec3 {
            glm::Vec3::new(x, y, z)
        };
        let vec2 = |x: f32, y: f32| -> glm::Vec2 {
            glm::Vec2::new(x, y)
        };
        let vertex = |pos: glm::Vec3, color: glm::Vec3, tex_coord: glm::Vec2| -> Vertex {
            Vertex::new(pos, color, tex_coord)
        };  
        
        let nb = edge as f32;
        let pi = std::f32::consts::PI;
        let mut unique_vertices : HashMap<Vertex, u32> = HashMap::new();
        let mut vertices : Vec<Vertex> = Vec::new();
        let mut indices : Vec<u32> = Vec::new();

		// //center of the circle
		let center = vertex(center.pos(), center.color(), vec2(0.5, 0.5));
		vertices.push(center);

		for i in 0..edge {
            let i = i as f32;
        	let v1x = f32::cos(i * 2.0 * pi / nb);
			let v1y = f32::sin(i * 2.0 * pi / nb);
			let v1 = vertex(
				center.pos() + vec3(radius * v1x , radius * v1y, 0.),
				center.color(),
				vec2((0.5 * v1x) + 0.5, 0.5 - (0.5 * v1y))
            );

            let v3x = f32::cos((i + 1.0) * 2.0 * pi / nb);
			let v3y = f32::sin((i + 1.0) * 2.0 * pi / nb);
			let v3 = vertex(
				center.pos()  + vec3(radius * v3x, radius * v3y, 0.0),
				center.color(),
				vec2((0.5 * v3x) + 0.5, 0.5 - (0.5 * v3y))
            );

            //add the vertex and the index if vertex is unique
			add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v1);
			indices.push(0);			//add the index of the center of the circle
			add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v3);
		}		

		for i in 0..indices.len() {
			indices.push(indices[i + 2]);
			indices.push(0);
			indices.push(indices[i]);
		}

        Self {
            vertices,
            indices,
            texture,
            fn_update_matrix: None,
        }
    }
}

impl Object for Circle {
    fn vertices(&self) -> &[Vertex] {
        self.vertices.as_ref()
    }

    fn indices (&self) -> &[u32] {
        self.indices.as_ref()
    }

    fn texture (&self) -> Option<Arc<Texture>> {
        self.texture.clone()
    }

    fn set_texture(&mut self, texture: Arc<Texture>) {
        self.texture = Some(texture);
    }

    fn set_fn_update_matrix(&mut self, f: fn(usize, f32, u32, u32) -> MatrixShaderObject) {
        self.fn_update_matrix = Some(f);
    }

    fn get_fn_update_matrix(&self) -> Option<fn(usize, f32, u32, u32) -> MatrixShaderObject> {
        self.fn_update_matrix
    }
}