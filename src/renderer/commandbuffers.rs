use vulkanalia::{
    prelude::v1_0::*
};
use std::time::Instant;
use nalgebra_glm as glm;
use anyhow::Result;
use crate::renderer::{
    appdata::AppData,
    queue_family::QueueFamilyIndices,
};

//================================================
// Command Pool
//================================================

pub unsafe fn create_command_pools(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {
    data.command_pool = create_command_pool(instance, device, data)?;

    let num_images = data.swapchain_images.len();
    for _ in 0..num_images {
        let command_pool = create_command_pool(instance, device, data)?;
        data.command_pools.push(command_pool);
    }

    Ok(())
}

unsafe fn create_command_pool(instance: &Instance, device: &Device, data: &mut AppData) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&info, None)?)
}

//================================================
// Command Buffers
//================================================

pub unsafe fn create_command_buffers(_device: &Device, data: &mut AppData) -> Result<()> {
    data.command_buffers = vec![vk::CommandBuffer::null(); data.framebuffers.len()];

    Ok(())
}

pub unsafe fn update_command_buffer(device: &Device, data: &mut AppData, 
    image_index: usize, count_models: usize, start: &Instant) -> Result<()> 
{
    // Reset
    let command_pool = data.command_pools[image_index];
    device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

    // Allocate
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];
    data.command_buffers[image_index] = command_buffer;

    //commands
    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    let render_area = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(data.swapchain_extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };
    let depth_clear_value = vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
    };

    let clear_values = &[color_clear_value, depth_clear_value];
    let info = vk::RenderPassBeginInfo::builder()
        .render_pass(data.render_pass)
        .framebuffer(data.framebuffers[image_index])
        .render_area(render_area)
        .clear_values(clear_values);

    device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);

    let secondary_command_buffers = (0..count_models)
        .map(|i| update_secondary_command_buffer(device, data, image_index, i, start))
        .collect::<Result<Vec<_>, _>>()?;
    device.cmd_execute_commands(command_buffer, &secondary_command_buffers[..]);

    device.cmd_end_render_pass(command_buffer);

    device.end_command_buffer(command_buffer)?;

    Ok(())
}

unsafe fn update_secondary_command_buffer(
    device : &Device,
    data: &AppData,
    image_index: usize,
    model_index: usize,
    start: &Instant,
) -> Result<vk::CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pools[image_index])
        .level(vk::CommandBufferLevel::SECONDARY)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

    let y = (((model_index % 2) as f32) * 2.5) - 1.25;
    let z = (((model_index / 2) as f32) * -2.0) + 1.0;
    let time = start.elapsed().as_secs_f32();

    let model = glm::translate(
        &glm::identity(),
        &glm::vec3(0.0, y, z),
    );    
    let model = glm::rotate(
        &model,
        time * glm::radians(&glm::vec1(90.0))[0],
        &glm::vec3(0.0, 0.0, 1.0),
    );
    let (_, model_bytes, _) = model.as_slice().align_to::<u8>();

    let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
        .render_pass(data.render_pass)
        .subpass(0)
        .framebuffer(data.framebuffers[image_index]);

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
        .inheritance_info(&inheritance_info);

    device.begin_command_buffer(command_buffer, &info)?;

    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);
    device.cmd_bind_vertex_buffers(command_buffer, 0, &[data.vertex_buffer], &[0]);
    device.cmd_bind_index_buffer(command_buffer, data.index_buffer, 0, vk::IndexType::UINT32);
    device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        data.pipeline_layout,
        0,
        &[data.descriptor_sets[image_index]],
        &[],
    );
    device.cmd_push_constants(
        command_buffer,
        data.pipeline_layout,
        vk::ShaderStageFlags::VERTEX,
        0,
        model_bytes,
    );
    device.cmd_draw_indexed(command_buffer, data.indices.len() as u32, 1, 0, 0, 0);

    device.end_command_buffer(command_buffer)?;

    Ok(command_buffer)
}
