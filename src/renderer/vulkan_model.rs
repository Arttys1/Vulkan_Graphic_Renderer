use std::sync::Arc;

use crate::tools::{texture::Texture, model::Model};

use super::renderer::Renderer;

use {
    anyhow::{Result, anyhow},
    vulkanalia::prelude::v1_0::*,
    super::{
        vulkan_texture::VulkanTexture,
        vertexbuffers::VertexBuffer, 
        uniformbuffers::UniformBuffer,
        descriptor::Descriptor,
    },
};

#[derive(Debug, Clone)]
pub struct VulkanModel {
    texture: VulkanTexture,
    buffer: VertexBuffer,
    uniform_buffer: UniformBuffer,
    descriptor: Descriptor,
}

impl VulkanModel {
    pub fn read(device: Arc<Device>, instance: &Instance, 
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout, model: Arc<Model>,
        texture: Arc<Texture>) -> Result<Self> 
    {
        let vertices = model.vertices();
        let indices = model.indices();
        
        let buffer = VertexBuffer::new(device.clone(), instance, physical_device, command_pool, graphics_queue, vertices, indices)?;
        let texture = VulkanTexture::new(device.clone(), instance, physical_device, command_pool, graphics_queue, texture)?;
        let mut uniform_buffer = UniformBuffer::new(device.clone(), instance,physical_device,swapchain_images)?;
        uniform_buffer.set_fn_update_ubo(Renderer::update_ubo);
        uniform_buffer.set_fn_update_push_constant(Renderer::update_push_constant);
        let descriptor = Descriptor::new(device.clone(),             
            swapchain_images,
            descriptor_set_layout, 
            &uniform_buffer, 
            &texture)?;
        Ok(VulkanModel {
            texture,
            buffer,
            uniform_buffer,
            descriptor,
        })
    }

    pub fn construct(buffer: VertexBuffer, texture: VulkanTexture, uniform_buffer: UniformBuffer, descriptor: Descriptor)
-> Result<Self> {
        if !(buffer.is_allocated() && texture.is_allocated() && uniform_buffer.is_allocated() && descriptor.is_allocated()) {
            return Err(anyhow!("vertex_buffer, texture and uniform_buffer must be allocated before"));
        }
        Ok(VulkanModel {
            texture,
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

    pub fn update(&mut self, device: &Device, swapchain_extent: vk::Extent2D, image_index: usize) -> Result<()> {
        unsafe {
            self.uniform_buffer.update_uniform_buffer(device, swapchain_extent, image_index)?;
            Ok(())
        }
    }

    pub fn texture(&self) -> &VulkanTexture {
        &self.texture
    }

    pub fn buffer(&self) -> &VertexBuffer {
        &self.buffer
    }

    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }

    pub fn uniform_buffer(&self) -> &UniformBuffer {
        &self.uniform_buffer
    }
}

impl Drop for VulkanModel {
    fn drop(&mut self) {
        self.clean();
    }
}