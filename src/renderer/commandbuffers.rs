use vulkanalia::{
    prelude::v1_0::*
};
use std::time::Instant;
use anyhow::Result;
use crate::renderer::{
    core::Core,
    queue_family::QueueFamilyIndices,
};

//================================================
// Command Pool
//================================================

pub unsafe fn create_command_pools(
    instance: &Instance, 
    device: &Device, 
    swapchain_images: &Vec<vk::Image>,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice)
-> Result<(vk::CommandPool, Vec<vk::CommandPool> )> {
    let command_pool = create_command_pool(instance, device, surface, physical_device)?;

    let mut command_pools = Vec::<vk::CommandPool>::default();
    for _ in 0..swapchain_images.len() {
        let command_pool = create_command_pool(instance, device, surface, physical_device)?;
        command_pools.push(command_pool);
    }

    Ok((command_pool, command_pools))
}

unsafe fn create_command_pool(
    instance: &Instance, 
    device: &Device, 
    surface: vk::SurfaceKHR, 
    physical_device: vk::PhysicalDevice)
-> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, surface, physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&info, None)?)
}

//================================================
// Command Buffers
//================================================


pub unsafe fn create_command_buffers(
    device: &Device, 
    swapchain_images: &Vec<vk::Image>, 
    command_pools: &Vec<vk::CommandPool>) -> Result<Vec<vk::CommandBuffer>> {
    let num_images = swapchain_images.len();
    let mut command_buffers : Vec<vk::CommandBuffer> = Vec::with_capacity(num_images);
    for image_index in 0..num_images {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pools[image_index])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];
        command_buffers.push(command_buffer);
    }
    Ok(command_buffers)
}

pub unsafe fn update_command_buffer(device: &Device, core: &mut Core, 
    image_index: usize, start: &Instant) -> Result<()> 
{
    // Reset
    let command_pool = core.command_pools()[image_index];
    device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;
    // Allocate
    let command_buffer = core.command_buffers()[image_index];

    //commands
    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    let render_area = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(core.swapchain_extent());

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.1, 0.1, 1.0],
        },
    };
    let depth_clear_value = vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
    };

    let clear_values = &[color_clear_value, depth_clear_value];
    let info = vk::RenderPassBeginInfo::builder()
        .render_pass(core.render_pass())
        .framebuffer(core.framebuffers()[image_index])
        .render_area(render_area)
        .clear_values(clear_values);

    device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);

    if !core.models().is_empty() {
        let secondary_command_buffers = (0..core.models().len())
            .map(|i| update_secondary_command_buffer(device, core, core.command_pools()[image_index], image_index, i, start))
            .collect::<Result<Vec<_>, _>>()?;
        device.cmd_execute_commands(command_buffer, &secondary_command_buffers[..]);
    }
    device.cmd_end_render_pass(command_buffer);

    device.end_command_buffer(command_buffer)?;

    Ok(())
}

unsafe fn update_secondary_command_buffer(
    device : &Device,
    core: &mut Core,
    command_pool: vk::CommandPool,
    image_index: usize,
    model_index: usize,
    start: &Instant,
) -> Result<vk::CommandBuffer> {
    let secondary_command_buffers = core.secondary_command_buffers_mut();
    let secondary_command_buffers = &mut secondary_command_buffers[image_index];
    while model_index >= secondary_command_buffers.len() {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::SECONDARY)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];
        secondary_command_buffers.push(command_buffer);
    }

    let command_buffer = secondary_command_buffers[model_index];

    //model who will be draw
    let model = core.at_model(model_index);
    let descriptor = &model.descriptor().descriptor_sets()[image_index];
    let model_buffer = model.buffer();
    let shader_ptr = model.shader();
    let shader = shader_ptr.borrow();

    //push constant data
    let elapsed_time = start.elapsed().as_secs_f32();
    let push_constant_object= model.uniform_buffer().update_matrix(
        device, 
        core.swapchain_extent(),
        image_index,
        model_index, 
        elapsed_time)?;

    let mat_model = push_constant_object.model();
    let view = push_constant_object.view();
    let model_slice = mat_model.as_slice();
    let view_slice = view.as_slice();
    let mut vec_push_constant = Vec::from(model_slice);
    vec_push_constant.append(&mut Vec::from(view_slice));
    let (_, push_constant_data, _) = vec_push_constant.as_slice().align_to::<u8>();

    //info command buffer
    let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
        .render_pass(core.render_pass())
        .subpass(0)
        .framebuffer(core.framebuffers()[image_index]);

    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
        .inheritance_info(&inheritance_info);
    
    device.begin_command_buffer(command_buffer, &info)?;

    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, shader.pipeline());
    device.cmd_bind_vertex_buffers(command_buffer, 0, &[model_buffer.vertex_buffer()], &[0]);
    device.cmd_bind_index_buffer(command_buffer, model_buffer.index_buffer(), 0, vk::IndexType::UINT32);

    device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        shader.pipeline_layout(),
        0,
        &[*descriptor],
        &[],
    );
    device.cmd_push_constants(
        command_buffer,
        shader.pipeline_layout(),
        vk::ShaderStageFlags::VERTEX,
        0,
        push_constant_data,
    );
    device.cmd_draw_indexed(command_buffer, model_buffer.indices_len() as u32, 1, 0, 0, 0);

    device.end_command_buffer(command_buffer)?;
    Ok(command_buffer)
}
