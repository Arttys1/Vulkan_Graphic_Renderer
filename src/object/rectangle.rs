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

pub struct Rectangle {
    vertices: [Vertex; 4],
    texture: Option<Arc<Texture>>,
    fn_update_matrix : Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>,
}

impl Rectangle {
    pub fn new(vertices: [Vertex; 4], texture: Option<Arc<Texture>>) -> Self {
        Self { vertices, texture, fn_update_matrix: None } 
    }
    pub fn from_one(mut one: Vertex, width: f32, height: f32, texture: Option<Arc<Texture>>) -> Self {
        one.set_tex_coord(glm::Vec2::new(0.0, 0.0));
        let two = Vertex::new(
            one.pos() + glm::Vec3::new(width, 0.0, 0.0),
            one.color(),
            glm::Vec2::new(1.0, 0.0),
        );
        let three = Vertex::new(
            one.pos() + glm::Vec3::new(0.0, height, 0.0),
            one.color(),
            glm::Vec2::new(0.0, 1.0),
        );
        let four = Vertex::new(
            one.pos() + glm::Vec3::new(width, height, 0.0),
            one.color(),
            glm::Vec2::new(1.0, 1.0),
        );
        Self {
            vertices: [one, two, three, four],
            texture,
            fn_update_matrix: None,
        }
    }  
}

impl Object for Rectangle {
    fn vertices(&self) -> &[Vertex] {
        self.vertices.as_ref()
    }

    fn indices (&self) -> &[u32] {
        &[0, 1, 2, 2, 3, 0, 	//one face 
          2, 1, 0, 0, 3, 2 ]    //behind
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