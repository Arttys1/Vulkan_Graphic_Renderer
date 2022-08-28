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