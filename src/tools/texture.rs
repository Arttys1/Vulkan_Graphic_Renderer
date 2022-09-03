use std::fs::File;

use super::loader::Loadable;
use anyhow::{Error, anyhow};

pub struct Texture {
    data: Vec<u8>, 
    info: png::OutputInfo,
}

impl Texture {
    pub(crate) fn construct(data: Vec<u8>, info: png::OutputInfo) -> Self {
        Self { data, info }
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }
    pub fn width(&self) -> u32 {
        self.info.width
    }
    pub fn height(&self) -> u32 {
        self.info.height
    }
    pub fn buffer_size(&self) -> usize {
        self.info.buffer_size()
    }
}

impl Loadable for Texture {
    fn load(path: &String) -> Result<Self, Error> {
        let image = File::open(path)?;
        let decoder = png::Decoder::new(image);
        let (info, mut reader) = match decoder.read_info() {
            Ok(tuple) => tuple,
            Err(e) => return Err(anyhow!(e))
        };

        let mut data = vec![0; info.buffer_size()];
        reader.next_frame(&mut data)?;
        Ok(Texture::construct(data, info))
    }
}