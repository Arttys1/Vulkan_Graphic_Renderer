use std::cell::RefCell;

use super::vulkan_shader::{VulkanShader, ShaderContainer, ShaderType};

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
    shader: Arc<RefCell<VulkanShader>>,
    texture: Option<VulkanTexture>,
    buffer: VertexBuffer,
    uniform_buffer: UniformBuffer,
    descriptor: Descriptor,
}

impl VulkanModel {
    pub fn from_obj(device: Arc<Device>, shader_container: &mut ShaderContainer, instance: &Instance, 
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, swapchain_images: &Vec<vk::Image>,
        swapchain_extent: vk::Extent2D, msaa_samples: vk::SampleCountFlags,
        render_pass: vk::RenderPass, obj: &dyn Object,) -> Result<Self> 
    {
        let vertices = &obj.vertices().to_vec();
        let indices = &obj.indices().to_vec();
        let shader : Arc<RefCell<VulkanShader>>;
        let vulkan_texture : Option<VulkanTexture>;
        if let Some(texture) = obj.texture() {
            shader = shader_container.get(ShaderType::Textured, swapchain_extent, msaa_samples, render_pass)?;
            vulkan_texture = Some(VulkanTexture::new(device.clone(), instance, physical_device, command_pool, graphics_queue, texture)?);
        } 
        else {
            shader = shader_container.get(ShaderType::Untextured, swapchain_extent, msaa_samples, render_pass)?;
            vulkan_texture = None;
        }
        let buffer = VertexBuffer::new(device.clone(), instance, physical_device, command_pool, graphics_queue, vertices, indices)?;
        let mut uniform_buffer = UniformBuffer::new(device.clone(), instance,physical_device,swapchain_images)?;
        if let Some(f) = obj.get_fn_update_matrix() {
            uniform_buffer.set_fn_update_matrix(f);
        }
        
        let descriptor = Descriptor::new(device.clone(),             
            swapchain_images,
            shader.borrow().descriptor_set_layout(), 
            &uniform_buffer, 
            &vulkan_texture)?;    
        Ok(VulkanModel {
            shader,
            texture: vulkan_texture,
            buffer,
            uniform_buffer,
            descriptor,
        })
    }

    pub fn clean(&mut self) {
        if let Some(texture) = &mut self.texture {
            texture.clean();
        } 
        self.buffer.clean();
        self.uniform_buffer.clean();
        self.descriptor.clean();  
    }

    pub fn reload_swapchain(&mut self,
        instance: &Instance, 
        physical_device: vk::PhysicalDevice,
        swapchain_images: &Vec<vk::Image>) -> Result<()> 
    {
        self.uniform_buffer.reload_swapchain_models(instance, physical_device, swapchain_images)?;
        self.descriptor.reload_swapchain(swapchain_images, self.shader().borrow().descriptor_set_layout(), &self.uniform_buffer, &self.texture)?;
        
        Ok(())
    }

    pub fn texture(&self) -> Option<VulkanTexture> { self.texture.clone() }
    pub fn buffer(&self) -> &VertexBuffer { &self.buffer }
    pub fn descriptor(&self) -> &Descriptor { &self.descriptor }
    pub fn uniform_buffer(&self) -> &UniformBuffer { &self.uniform_buffer }

    pub fn shader(&self) -> Arc<RefCell<VulkanShader>> {
        self.shader.clone()
    }
}

impl Drop for VulkanModel {
    fn drop(&mut self) {
        self.clean();
    }
}