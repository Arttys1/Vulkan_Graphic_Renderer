use {
    std::{sync::Arc},
    crate::{
        renderer::{vertex::Vertex, uniformbuffers::MatrixShaderObject},
        tools::{texture::Texture, model::Model},        
    },
    super::Object,
};

pub struct Mesh {
    model: Arc<Model>,
    texture: Option<Arc<Texture>>,
    fn_update_matrix : Option<fn(usize, f32, u32, u32) -> MatrixShaderObject>,
}

impl Mesh {
    pub fn new(model: Arc<Model>, texture: Option<Arc<Texture>>) -> Self {
        Self { model, texture, fn_update_matrix: None }
    }
    pub fn construct(vertices: Vec<Vertex>, indices: Vec<u32>, texture :Option<Arc<Texture>>) -> Self {
        let model = Arc::new(Model::construct(vertices, indices));
        Self { model, texture, fn_update_matrix: None }
    }
}

impl Object for Mesh {
    fn vertices(&self) -> &[Vertex] {
        self.model.vertices()
    }

    fn indices (&self) -> &[u32] {
        self.model.indices()
    }

    fn texture (&self) -> Option<Arc<Texture>> {
        self.texture.clone()
    }

    fn set_fn_update_matrix(&mut self, f: fn(usize, f32, u32, u32) -> MatrixShaderObject) {
        self.fn_update_matrix = Some(f);
    }

    fn get_fn_update_matrix(&self) -> Option<fn(usize, f32, u32, u32) -> MatrixShaderObject> {
        self.fn_update_matrix
    }
}


