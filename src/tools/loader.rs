use { 
    std::{
        fs::File, 
        collections::HashMap, 
        sync::Arc
    },
    anyhow::{Error, anyhow},
    super::{
        texture::Texture,
        model::Model, 
    },
};

pub struct Loader {
    texture_loaded: HashMap<String, Arc<Texture>>,
    model_loaded: HashMap<String, Arc<Model>>,
}

impl Default for Loader {
    fn default() -> Self {
        Self {
            texture_loaded: HashMap::default(),
            model_loaded: HashMap::default(),
        }
    }
}

impl Loader{
    pub fn load_texture(&mut self, path: &String) -> Result<Arc<Texture>, Error>{        
        if let Some(texture) = self.texture_loaded.get(path) {
            Ok(texture.clone()) //case where texture is already loaded
        }
        else {      //case where texture is not in the hashmap
            // Load
            let image = File::open(path)?;

            let decoder = png::Decoder::new(image);
            let (info, mut reader) = match decoder.read_info() {
                Ok(tuple) => tuple,
                Err(e) => return Err(anyhow!(e))
            };

            let mut data = vec![0; info.buffer_size()];
            reader.next_frame(&mut data)?;
            
            let texture = Arc::new(Texture::construct(data, info));
            self.texture_loaded.insert(path.clone(), texture.clone());
            Ok(texture.clone())
        }
    }

    pub fn load_model(&mut self, path: &String) -> Result<Arc<Model>, Error> {
        if let Some(model) = self.model_loaded.get(path) {
            Ok(model.clone())   //case where model is already loaded
        }
        else {      //case where we need to load a new model
            
            let model = Arc::new(Model::new(path)?);
            self.model_loaded.insert(path.clone(), model.clone());
            Ok(model.clone())
        }
    }
}
