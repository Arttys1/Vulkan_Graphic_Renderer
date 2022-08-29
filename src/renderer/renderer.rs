use {
    std::{time::Instant, sync::Arc},
    vulkanalia::{
        loader::{ LibloadingLoader, LIBRARY },
        prelude::v1_0::*,
        vk::KhrSwapchainExtension,
    },
    winit::window::Window,
    anyhow::{anyhow, Result},
    crate::object::Object,
    super::{
        appdata::*,
        commandbuffers::*, 
        vulkan_model::VulkanModel,
    },
};
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct Renderer {
    _entry: Entry,
    device: Arc<Device>,
    data: AppData,
    frame: usize,
    resized: bool,
    start: Instant,
}

impl Renderer {
    /// Creates our Vulkan app.
    pub fn create(window: &Window) -> Result<Self> {
        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
            let data = AppData::new(window, &entry)?;

            let renderer = Self { 
                _entry: entry,
                device: data.device(),
                data,
                frame: 0, 
                resized: false, 
                start: Instant::now(), 
            };
            Ok(renderer)
        }
    }

    /// Renders a frame for our Vulkan app.
    pub fn render(&mut self, window: &Window) -> Result<()> {
        unsafe {
            let in_flight_fence = self.data.in_flight_fences()[self.frame];

            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;

            let image_index = self
                .device
                .acquire_next_image_khr(
                    self.data.swapchain(),
                    u64::max_value(),
                    self.data.image_available_semaphores()[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize;

            let image_in_flight = self.data.images_in_flight()[image_index];
            if !image_in_flight.is_null() {
                self.device
                    .wait_for_fences(&[image_in_flight], true, u64::max_value())?;
            }

            self.data.images_in_flight_mut()[image_index] = in_flight_fence;

            update_command_buffer(&self.device, &mut self.data, image_index, &self.start)?;

            let wait_semaphores = &[self.data.image_available_semaphores()[self.frame]];
            let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[self.data.command_buffers()[image_index]];
            let signal_semaphores = &[self.data.render_finished_semaphores()[self.frame]];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);

            self.device.reset_fences(&[in_flight_fence])?;

            self.device
                .queue_submit(self.data.graphics_queue(), &[submit_info], in_flight_fence)?;

            let swapchains = &[self.data.swapchain()];
            let image_indices = &[image_index as u32];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .swapchains(swapchains)
                .image_indices(image_indices);

            let result = self.device.queue_present_khr(self.data.present_queue(), &present_info);
            let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
            if self.resized || changed {
                self.resized = false;
                self.data.recreate_swapchain(window)?;
            } else if let Err(e) = result {
                return Err(anyhow!(e));
            }
                

            self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

            Ok(())
        }
    } 

    pub fn add_object(&mut self, obj: &dyn Object) -> Result<()> {
        let model = VulkanModel::from_obj(
            self.data.device(),
            self.data.instance(),
            self.data.physical_device(),
            self.data.command_pool(),
            self.data.graphics_queue(),
            self.data.swapchain_images(),
            self.data.descriptor_set_layout(),
            obj)?;
        self.data.push_model(model);
        Ok(())
    }

    pub fn clean(&mut self) {
        self.data.clean();
    }

    pub fn must_resize(&mut self) {
        self.resized = true;
    } 
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.clean();
    }   
}
