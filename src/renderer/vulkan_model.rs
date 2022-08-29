use {
    std::sync::Arc,
    anyhow::Result,
    vulkanalia::prelude::v1_0::*,
    super::{
        vulkan_texture::VulkanTexture,
        vertexbuffers::VertexBuffer, 
        uniformbuffers::UniformBuffer,
        descriptor::Descriptor,
    },
    crate::object::Object,
};

#[derive(Debug, Clone)]
pub struct VulkanModel {
    texture: VulkanTexture,
    buffer: VertexBuffer,
    uniform_buffer: UniformBuffer,
    descriptor: Descriptor,
}

impl VulkanModel {
    pub fn from_obj(device: Arc<Device>, instance: &Instance, 
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout, obj: &dyn Object,) -> Result<Self> 
    {
        let vertices = &obj.vertices().to_vec();
        let indices = &obj.indices().to_vec();
        let texture = obj.texture().unwrap();

        let buffer = VertexBuffer::new(device.clone(), instance, physical_device, command_pool, graphics_queue, vertices, indices)?;
        let vulkan_texture = VulkanTexture::new(device.clone(), instance, physical_device, command_pool, graphics_queue, texture)?;
        let mut uniform_buffer = UniformBuffer::new(device.clone(), instance,physical_device,swapchain_images)?;
        if let Some(f) = obj.get_fn_update_matrix() {
            uniform_buffer.set_fn_update_matrix(f);
        }
        let descriptor = Descriptor::new(device.clone(),             
            swapchain_images,
            descriptor_set_layout, 
            &uniform_buffer, 
            &vulkan_texture)?;
        Ok(VulkanModel {
            texture: vulkan_texture,
            buffer,
            uniform_buffer,
            descriptor,
        })
    }

    pub fn clean(&mut self) {
        self.texture.clean();
        self.buffer.clean();
        self.uniform_buffer.clean();
        self.descriptor.clean();
    }

    pub fn reload_swapchain(&mut self,
        instance: &Instance, 
        physical_device: vk::PhysicalDevice,
        swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout) -> Result<()> 
    {
        self.uniform_buffer.reload_swapchain_models(instance, physical_device, swapchain_images)?;
        self.descriptor.reload_swapchain(swapchain_images, descriptor_set_layout, &self.uniform_buffer, &self.texture)?;
        Ok(())
    }

    pub fn texture(&self) -> &VulkanTexture { &self.texture }
    pub fn buffer(&self) -> &VertexBuffer { &self.buffer }
    pub fn descriptor(&self) -> &Descriptor { &self.descriptor }
    pub fn uniform_buffer(&self) -> &UniformBuffer { &self.uniform_buffer }
}

impl Drop for VulkanModel {
    fn drop(&mut self) {
        self.clean();
    }
}