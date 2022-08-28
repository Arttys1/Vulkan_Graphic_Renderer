use {
    std::{
        mem::size_of,
        sync::Arc,
    },
    vulkanalia::prelude::v1_0::*,
    anyhow::Result,
    super::{
        uniformbuffers::{
            UniformBuffer, 
            UniformBufferObject
        },
         vulkan_texture::VulkanTexture,
    },
};

#[derive(Debug, Clone)]
pub struct Descriptor {    
    device : Arc<Device>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    is_allocated: bool,
}

impl Descriptor {
    pub fn new(
        device: Arc<Device>,
        swapchain_images: &Vec<vk::Image>, 
        descriptor_set_layout: vk::DescriptorSetLayout, 
        uniform_buffers: &UniformBuffer,
        texture: &VulkanTexture,) -> Result<Self> 
    {
        unsafe {
            let descriptor_pool = create_descriptor_pool(&device, swapchain_images)?;
            let descriptor_sets = create_descriptor_sets(
                &device,
                swapchain_images,
                descriptor_set_layout,
                uniform_buffers.uniform_buffers(),
                descriptor_pool,
                texture.texture_image_view(),
                texture.texture_sampler())?;

                Ok(Descriptor{
                    device,
                    descriptor_pool,
                    descriptor_sets,
                    is_allocated: true,
                })
        }
    }

    pub fn reload_swapchain(&mut self,
        swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffer: &UniformBuffer,
        texture: &VulkanTexture,
    ) -> Result<()> {
        unsafe {
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.descriptor_pool = create_descriptor_pool(&self.device, swapchain_images)?;
            self.descriptor_sets = create_descriptor_sets(
                &self.device,
                swapchain_images,
                descriptor_set_layout,
                uniform_buffer.uniform_buffers(),
                self.descriptor_pool,
                texture.texture_image_view(),
                texture.texture_sampler())?;
            Ok(())
        }
    }


    pub fn clean(&mut self) {
        if self.is_allocated {
            unsafe {
                self.device.destroy_descriptor_pool(self.descriptor_pool, None);
                self.is_allocated = false;
            }
        }
    }

    pub(crate) fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    pub fn descriptor_sets(&self) -> &[vk::DescriptorSet] {
        self.descriptor_sets.as_ref()
    }
}

impl Drop for Descriptor {
    fn drop(&mut self) {
        self.clean();
    }
}

//================================================
// descriptor set layout
//================================================

pub unsafe fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    let descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(descriptor_set_layout)
}

//================================================
// descriptor pool
//================================================

pub unsafe fn create_descriptor_pool(device: &Device, swapchain_images: &Vec<vk::Image>) -> Result<vk::DescriptorPool> {
    let swapchain_len = swapchain_images.len() as u32;
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(swapchain_len);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(swapchain_len);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(swapchain_len);

    Ok(device.create_descriptor_pool(&info, None)?)
}


//================================================
// descriptor sets
//================================================

pub unsafe fn create_descriptor_sets(
        device: &Device, 
        swapchain_images: &Vec<vk::Image>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>,
        descriptor_pool: vk::DescriptorPool,
        image_view : vk::ImageView,
        sampler: vk::Sampler) -> Result<Vec<vk::DescriptorSet>> 
{
    let layouts = vec![descriptor_set_layout; swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets = device.allocate_descriptor_sets(&info)?;
    
    // Update
    for i in 0..swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
    }
    
    Ok(descriptor_sets)
}
