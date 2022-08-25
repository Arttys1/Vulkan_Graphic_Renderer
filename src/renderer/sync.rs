use vulkanalia::{
    prelude::v1_0::*
};
use anyhow::{Result};

use crate::renderer::{
    renderer::MAX_FRAMES_IN_FLIGHT,
};

//================================================
// Sync objects
//================================================

pub unsafe fn create_sync_objects(device: &Device, swapchain_images: &Vec<vk::Image>)
-> Result<(Vec<vk::Fence>, Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>)> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    let mut in_flight_fences =  Vec::<vk::Fence>::default();
    let mut render_finished_semaphores =  Vec::<vk::Semaphore>::default();
    let mut image_available_semaphores =  Vec::<vk::Semaphore>::default();
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        image_available_semaphores.push(device.create_semaphore(&semaphore_info, None)?);
        render_finished_semaphores.push(device.create_semaphore(&semaphore_info, None)?);
        in_flight_fences.push(device.create_fence(&fence_info, None)?);
    }

    let images_in_flight = swapchain_images.iter().map(|_| vk::Fence::null()).collect();

    Ok((in_flight_fences, render_finished_semaphores, image_available_semaphores, images_in_flight))
}