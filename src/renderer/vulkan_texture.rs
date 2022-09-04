use crate::tools::texture::Texture;

use {
    std::sync::Arc,
    vulkanalia::prelude::v1_0::*,
    anyhow::Result,
    super::{
        image::*,
    }
};
#[derive(Debug, Clone)]
pub struct VulkanTexture {
    device: Arc<Device>,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    is_allocated: bool,
}

impl VulkanTexture {
    pub fn empty(device: Arc<Device>) -> Result<Self> {
        Ok(VulkanTexture {
            device,
            texture_image: vk::Image::default(),
            texture_image_memory: vk::DeviceMemory::default(),
            texture_image_view: vk::ImageView::default(),
            texture_sampler: vk::Sampler::default(),
            is_allocated: false,
        })
    }

    pub fn new(device: Arc<Device>,instance: &Instance,
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, texture: Arc<Texture>) -> Result<Self>
    {
        unsafe {
            let (texture_image,
                texture_image_memory,
                mip_levels
            ) = create_texture_image(instance, &device, physical_device, command_pool, graphics_queue, texture)?;
            let texture_image_view = create_texture_image_view(&device, texture_image, mip_levels)?;
            let texture_sampler = create_texture_sampler(&device, mip_levels)?;
        
            Ok(VulkanTexture {
                device,
                texture_image,
                texture_image_memory,
                texture_image_view,
                texture_sampler,
                is_allocated: true,
            })
        }        
    }

    pub fn clean(&mut self) {
        if self.is_allocated {
            unsafe {
                self.device.destroy_sampler(self.texture_sampler, None);
                self.device.destroy_image_view(self.texture_image_view, None);
                self.device.destroy_image(self.texture_image, None);
                self.device.free_memory(self.texture_image_memory, None);
                self.is_allocated = false;
            }
        }
    }

    pub fn texture_image(&self) -> vk::Image { self.texture_image }
    pub fn texture_image_memory(&self) -> vk::DeviceMemory { self.texture_image_memory }
    pub fn texture_image_view(&self) -> vk::ImageView { self.texture_image_view }
    pub fn texture_sampler(&self) -> vk::Sampler { self.texture_sampler }
    
}

impl Drop for VulkanTexture {
    fn drop(&mut self) {
        self.clean();
    }
}
