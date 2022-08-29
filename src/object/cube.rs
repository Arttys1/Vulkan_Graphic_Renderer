use {
    std::sync::Arc,
    crate::{
        tools::texture::Texture,
        renderer::{
            vertex::Vertex, uniformbuffers::MatrixShaderObject
        }
    },
    super::Object,
    nalgebra_glm as glm,
};

pub struct Cube {
    vertices: [Vertex; 24],
    texture: Option<Arc<Texture>>,
    fn_update_matrix : Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>,
}

impl Cube {
    pub fn new(vertices: [Vertex; 24], texture: Option<Arc<Texture>>) -> Self {
        Self { vertices, texture, fn_update_matrix: None } 
    }
    pub fn from_one(one: Vertex, width: f32, height: f32, depth: f32, texture: Option<Arc<Texture>>) -> Self {
        let vec3 = |x: f32, y: f32, z: f32| -> glm::Vec3 {
            glm::Vec3::new(x, y, z)
        };
        let vec2 = |x: f32, y: f32| -> glm::Vec2 {
            glm::Vec2::new(x, y)
        };
        let vertex = |pos: glm::Vec3, color: glm::Vec3, tex_coord: glm::Vec2| -> Vertex {
            Vertex::new(pos, color, tex_coord)
        };        
        let pos = one.pos();  
        let color = one.color();

        let v0 = vertex(pos, color, vec2(0.0, 0.0)); //front
		let v1 = vertex(pos + vec3(width, 0.0, 0.0), color, vec2(1.0, 0.0));
		let v2 = vertex(pos + vec3(width, -height, 0.0), color , vec2(1.0, 1.0));
		let v3 = vertex(pos + vec3(0.0, -height, 0.0), color, vec2( 0.0, 1.0));

		let v4 = vertex(pos + vec3(0.0, 0.0, depth), color, vec2(1.0, 0.0) );	//back
		let v5 = vertex(pos + vec3(width, 0.0, depth), color, vec2(0.0, 0.0));
		let v6 = vertex(pos + vec3(width, -height, depth), color, vec2(0.0, 1.0));
		let v7 = vertex(pos + vec3(0.0, -height, depth), color, vec2(1.0, 1.0));
        
		let mut l0 = v0;		//left
		let mut l1 = v3;
		let mut l2 = v4;
		let mut l3 = v7;

		let mut r0 = v1;		//right
		let mut r1 = v2;
		let mut r2 = v5;
		let mut r3 = v6;

		let mut u0 = v0;		//up
		let mut u1 = v1;	
		let mut u2 = v4;	
		let mut u3 = v5;

		let mut d0 = v3;		//down
		let mut d1 = v2;
		let mut d2 = v6;
		let mut d3 = v7;

        l0.set_tex_coord(vec2( 1.0, 0.0 )); //texture coord
		l1.set_tex_coord(vec2( 1.0, 1.0 ));
		l2.set_tex_coord(vec2( 0.0, 0.0 ));
		l3.set_tex_coord(vec2( 0.0, 1.0 ));

		r0.set_tex_coord(vec2( 0.0, 0.0 ));
		r1.set_tex_coord(vec2( 0.0, 1.0 ));
		r2.set_tex_coord(vec2( 1.0, 0.0 ));
		r3.set_tex_coord(vec2( 1.0, 1.0 ));

		u0.set_tex_coord(vec2( 0.0, 1.0 ));
		u1.set_tex_coord(vec2( 1.0, 1.0 ));
		u2.set_tex_coord(vec2( 0.0, 0.0 ));
		u3.set_tex_coord(vec2( 1.0, 0.0 ));

		d0.set_tex_coord(vec2( 0.0, 0.0 ));
		d1.set_tex_coord(vec2( 1.0, 0.0 ));
		d2.set_tex_coord(vec2( 1.0, 1.0 ));
		d3.set_tex_coord(vec2( 0.0, 1.0 ));
        
        Self {
            vertices: 
            [v0, v1, v2, v3, v4, v5, v6, v7,
             l0, l1, l2, l3, r0, r1, r2, r3,
             u0, u1, u2, u3, d0, d1, d2, d3, ],
            texture,
            fn_update_matrix: None,
        }
    }  
}

impl Object for Cube {
    fn vertices(&self) -> &[Vertex] {
        self.vertices.as_ref()
    }

    fn indices (&self) -> &[u32] {
        &[0, 1, 3, // front
          1, 2, 3,
          6, 5, 4, //back
          4, 7, 6,	
          8, 9, 10, //left
          9, 11, 10,
          14, 13, 12, //right
          14, 15, 13,
          18, 17, 16, //up
          18, 19, 17,
          20, 21, 22, //down
          22, 23, 20,]
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