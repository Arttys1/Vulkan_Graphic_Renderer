use vulkanalia::{
    prelude::v1_0::*, 
};
use anyhow::{Result, anyhow};
use crate::renderer::{
    image::*,
};

//================================================
// Depth buffer
//================================================

pub unsafe fn create_depth_objects(
    instance: &Instance,
    device: &Device, 
    physical_device: vk::PhysicalDevice,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags)
-> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
    // Image + Image Memory
    let format = get_depth_format(instance, physical_device)?;

    let (depth_image, depth_image_memory) = create_image(
        instance,
        device,
        physical_device,
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        msaa_samples,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // Image View
    let depth_image_view = create_image_view(
        device,
        depth_image,
        format,
        vk::ImageAspectFlags::DEPTH,
        1,
    )?;

    Ok((depth_image, depth_image_memory, depth_image_view))
}

pub unsafe fn get_depth_format(instance: &Instance, physical_device: vk::PhysicalDevice,
) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance,
        physical_device,
        candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

unsafe fn get_supported_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance.get_physical_device_format_properties(physical_device, *f);
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
        .ok_or_else(|| anyhow!("Failed to find supported format!"))
}