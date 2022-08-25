use vulkanalia::{
    prelude::v1_0::*
};
use anyhow::Result;


//================================================
// Framebuffers
//================================================

pub unsafe fn create_framebuffers(
    device: &Device,
    swapchain_image_views: &Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    depth_image_view: vk::ImageView,
    color_image_view: vk::ImageView,) 
-> Result<Vec<vk::Framebuffer>> {
    let framebuffers = swapchain_image_views.iter()
        .map(|i| {
            let attachments = &[color_image_view, depth_image_view, *i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}