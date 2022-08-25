use {
    std::mem::size_of,
    vulkanalia::prelude::v1_0::*,
    anyhow::Result,
    super::{
        uniformbuffers::UniformBufferObject,
    }
};

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

pub unsafe fn load_descriptor_pool(device: &Device, swapchain_images: &Vec<vk::Image>) -> Result<vk::DescriptorPool> {
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

pub unsafe fn load_descriptor_sets(
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
