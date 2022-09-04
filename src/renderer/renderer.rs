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
        core::*,
        commandbuffers::*, 
    },
};
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Our Vulkan app.
#[derive(Clone)]
pub struct Renderer {
    _entry: Entry,
    device: Arc<Device>,
    core: Core,
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
            let core = Core::new(window, &entry)?;

            let renderer = Self { 
                _entry: entry,
                device: core.device(),
                core,
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
            let in_flight_fence = self.core.in_flight_fences()[self.frame];

            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;

            let image_index = self
                .device
                .acquire_next_image_khr(
                    self.core.swapchain(),
                    u64::max_value(),
                    self.core.image_available_semaphores()[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize;

            let image_in_flight = self.core.images_in_flight()[image_index];
            if !image_in_flight.is_null() {
                self.device
                    .wait_for_fences(&[image_in_flight], true, u64::max_value())?;
            }

            self.core.images_in_flight_mut()[image_index] = in_flight_fence;

            update_command_buffer(&self.device, &mut self.core, image_index, &self.start)?;

            let wait_semaphores = &[self.core.image_available_semaphores()[self.frame]];
            let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[self.core.command_buffers()[image_index]];
            let signal_semaphores = &[self.core.render_finished_semaphores()[self.frame]];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);

            self.device.reset_fences(&[in_flight_fence])?;

            self.device
                .queue_submit(self.core.graphics_queue(), &[submit_info], in_flight_fence)?;

            let swapchains = &[self.core.swapchain()];
            let image_indices = &[image_index as u32];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .swapchains(swapchains)
                .image_indices(image_indices);

            let result = self.device.queue_present_khr(self.core.present_queue(), &present_info);
            let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
            if self.resized || changed {
                self.resized = false;
                self.core.recreate_swapchain(window)?;
            } else if let Err(e) = result {
                return Err(anyhow!(e));
            }
                

            self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

            Ok(())
        }
    } 

    pub fn add_object(&mut self, obj: &dyn Object) -> Result<()> {
        unsafe {
            self.core.add_object(obj)?;        
            Ok(())
        }
    }

    pub fn clean(&mut self) {
        self.core.clean();
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
