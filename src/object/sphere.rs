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

pub struct Sphere {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,    
    texture: Option<Arc<Texture>>,
    fn_update_matrix : Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>,
}

impl Sphere {
    pub fn new(center: Vertex, latitudes: u8, longitudes: u8, texture: Option<Arc<Texture>>) -> Self {
        let vec2 = |x: f32, y: f32| -> glm::Vec2 {
            glm::Vec2::new(x, y)
        };
        let vertex = |pos: glm::Vec3, color: glm::Vec3, tex_coord: glm::Vec2| -> Vertex {
            Vertex::new(pos, color, tex_coord)
        };  
        let spherical = |norm: f32, theta: f32, phi: f32| -> glm::Vec3 {
            glm::Vec3::new(
            norm * theta.sin() * phi.sin(),
            norm * phi.cos(), 
            norm * theta.cos() * phi.sin())
        };
        
        let nb_y = longitudes as f32;
		let nb_x = latitudes as f32;
        let pi = std::f32::consts::PI;
        let mut unique_vertices : HashMap<Vertex, u32> = HashMap::new();
        let mut vertices : Vec<Vertex> = Vec::new();
        let mut indices : Vec<u32> = Vec::new();

		for x in 0..longitudes {
            let x = x as f32;
			for y in 0..latitudes
			{
                let y = y as f32;
				let v1 = vertex(
                    spherical(0.5, (x + 1.0) * 2. * pi / nb_x, y * pi / nb_y),
                    center.color(),			
                    vec2((x + 1.0) / nb_x, y / nb_y) );

				let v2 = vertex( spherical(0.5, x * 2.0 * pi / nb_x, y * pi / nb_y),
                    center.color(),
                    vec2(x / nb_x, y / nb_y ) );

				let v3 = vertex( spherical(0.5, x * 2.0 * pi / nb_x, (y + 1.0) * pi / nb_y),
                    center.color(),
                    vec2(x / nb_x, (y + 1.0) / nb_y) );

				let v4 = vertex( spherical(0.5, (x + 1.0) * 2.0 * pi / nb_x, y * pi / nb_y),
                    center.color(),
                    vec2((x + 1.0) / nb_x, y / nb_y) );

				let v5 = vertex( spherical(0.5, x * 2.0 * pi / nb_x, (y + 1.0) * pi / nb_y),
                    center.color(),
                    vec2(x / nb_x, (y + 1.0) / nb_y) );

				let v6 = vertex( spherical(0.5, (x + 1.0) * 2.0 * pi / nb_x, (y + 1.0) * pi / nb_y),
                    center.color(),
                    vec2((x + 1.0) / nb_x, (y + 1.0) / nb_y) );


                add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v1);
                add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v2);
                add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v3);
                add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v4);
                add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v5);
                add_unique_vertex(&mut unique_vertices, &mut vertices, &mut indices, v6);

			}
        }

        Self {
            vertices, indices, texture, fn_update_matrix: None,
        }
    }
}

impl Object for Sphere {
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