use {
    std::sync::Arc,
    vulkanalia::prelude::v1_0::*,
    anyhow::Result,
    super::{
        image::*,
        renderer::Renderer,
        descriptor::{
            load_descriptor_sets,
            load_descriptor_pool
        }
    }
};
#[derive(Debug, Clone)]
pub struct Texture {
    device: Arc<Device>,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    is_allocated: bool,
}

impl Texture {
    pub fn empty(renderer: &Renderer) -> Result<Self> {
        Ok(Texture {
            device: renderer.get_device(),
            texture_image: vk::Image::default(),
            texture_image_memory: vk::DeviceMemory::default(),
            texture_image_view: vk::ImageView::default(),
            texture_sampler: vk::Sampler::default(),
            descriptor_pool: vk::DescriptorPool::default(),
            descriptor_sets: Vec::default(),
            is_allocated: false,
        })
    }

    pub fn new(renderer: &Renderer, url: &str) -> Result<Self> {
        let data = renderer.get_appdata();
        let device = renderer.get_device();
        unsafe {            
            let (texture_image,
                texture_image_memory,
                mip_levels
            ) = load_texture_image(renderer.get_instance(), &device, data.physical_device(), data.command_pool(), data.graphics_queue(), url)?;
            let texture_image_view = load_texture_image_view(&device, texture_image, mip_levels)?;
            let texture_sampler = load_texture_sampler(&device, mip_levels)?;
            let descriptor_pool = load_descriptor_pool(&device, data.swapchain_images())?;
            let descriptor_sets = load_descriptor_sets(
                &device,
                data.swapchain_images(),
                data.descriptor_set_layout(),
                data.uniform_buffers(),
                descriptor_pool,
                texture_image_view,
                texture_sampler)?;
                    
            Ok(Texture {
                device,
                texture_image,
                texture_image_memory,
                texture_image_view,
                texture_sampler,
                descriptor_pool,
                descriptor_sets,
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
                self.device.destroy_descriptor_pool(self.descriptor_pool, None);
                self.is_allocated = false;
            }
        }
    }

    pub fn reload_swapchain(&mut self,
        swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>,) -> Result<()> {
        unsafe {
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.descriptor_pool = load_descriptor_pool(&self.device, swapchain_images)?;
            self.descriptor_sets = load_descriptor_sets(
                &self.device,
                swapchain_images,
                descriptor_set_layout,
                uniform_buffers,
                self.descriptor_pool,
                self.texture_image_view,
                self.texture_sampler)?;
            Ok(())
        }
    }

    pub fn texture_image(&self) -> vk::Image { self.texture_image }
    pub fn texture_image_memory(&self) -> vk::DeviceMemory { self.texture_image_memory }
    pub fn texture_image_view(&self) -> vk::ImageView { self.texture_image_view }
    pub fn texture_sampler(&self) -> vk::Sampler { self.texture_sampler }
    pub fn descriptor_pool(&self) -> vk::DescriptorPool { self.descriptor_pool }
    pub fn descriptor_sets(&self) -> &[vk::DescriptorSet] { self.descriptor_sets.as_ref() }

    pub fn is_allocated(&self) -> bool {
        self.is_allocated
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.clean();
    }
}
