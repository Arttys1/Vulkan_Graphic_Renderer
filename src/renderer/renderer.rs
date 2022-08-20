use std::time::Instant;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY,},
    prelude::v1_0::*,
    vk::{
        KhrSurfaceExtension,
        KhrSwapchainExtension,
        ExtDebugUtilsExtension,
    },
    window as vk_window,
};
use winit::window::Window;
use anyhow::{anyhow, Result};
use crate::renderer::{
    appdata::*,
    instance::*,
    pipeline::*,
    queue_family::*,
    framebuffers::*,
    swapchain::*,
    commandbuffers::*,
    sync::*, 
    vertexbuffers::*,
    vertex::load_model,
    descriptor::*,
    image::*, 
    depthbuffers::create_depth_objects,
};

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct Renderer {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
    frame: usize,
    resized: bool,
    start: Instant,
    pub models: usize,
}

impl Renderer {
    /// Creates our Vulkan app.
    pub fn create(window: &Window) -> Result<Self> {
        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
            let mut data = AppData::default();

            let instance = create_instance(window, &entry, &mut data)?;
            data.surface = vk_window::create_surface(&instance, window)?;

            pick_physical_device(&instance, &mut data).expect("No physical device found.");
            let device = create_logical_device(&instance, &mut data)?;

            create_swapchain(window, &instance, &device, &mut data)?;
            create_swapchain_image_views(&device, &mut data)?;

            create_render_pass(&instance, &device, &mut data)?;
            create_descriptor_set_layout(&device, &mut data)?;
            create_pipeline(&device, &mut data)?;
            create_command_pools(&instance, &device, &mut data)?;
            create_color_objects(&instance, &device, &mut data)?;
            create_depth_objects(&instance, &device, &mut data)?;
            create_framebuffers(&device, &mut data)?;
            create_texture_image(&instance, &device, &mut data)?;
            create_texture_image_view(&device, &mut data)?;
            create_texture_sampler(&device, &mut data)?;
            load_model(&mut data)?;
            create_vertex_buffer(&instance, &device, &mut data)?;
            create_index_buffer(&instance, &device, &mut data)?;
            create_uniform_buffers(&instance, &device, &mut data)?;
            create_descriptor_pool(&device, &mut data)?;
            create_descriptor_sets(&device, &mut data)?;
            create_command_buffers(&device, &mut data)?;
            create_sync_objects(&device, &mut data)?;

            let renderer = Self { 
                entry,
                instance,
                data,
                device,
                frame: 0, 
                resized: false, 
                start: Instant::now(), 
                models: 1,
            };
            Ok(renderer)
        }
    }

    /// Renders a frame for our Vulkan app.
    pub fn render(&mut self, window: &Window) -> Result<()> {
        unsafe {
            let in_flight_fence = self.data.in_flight_fences[self.frame];

            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;

            let image_index = self
                .device
                .acquire_next_image_khr(
                    self.data.swapchain,
                    u64::max_value(),
                    self.data.image_available_semaphores[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize;

            let image_in_flight = self.data.images_in_flight[image_index];
            if !image_in_flight.is_null() {
                self.device
                    .wait_for_fences(&[image_in_flight], true, u64::max_value())?;
            }

            self.data.images_in_flight[image_index] = in_flight_fence;

            update_command_buffer(&self.device, &mut self.data, image_index, self.models, &self.start)?;
            update_uniform_buffer(&self.device, &self.data, image_index)?;

            let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
            let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[self.data.command_buffers[image_index]];
            let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);

            self.device.reset_fences(&[in_flight_fence])?;

            self.device
                .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?;

            let swapchains = &[self.data.swapchain];
            let image_indices = &[image_index as u32];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .swapchains(swapchains)
                .image_indices(image_indices);

            let result = self.device.queue_present_khr(self.data.present_queue, &present_info);
            let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
            if self.resized || changed {
                self.resized = false;
                self.recreate_swapchain(window)?;
            } else if let Err(e) = result {
                return Err(anyhow!(e));
            }
                

            self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

            Ok(())
        }
    }
    

    /// Destroys our Vulkan app.
    pub fn clean(&mut self) {
        unsafe {
            //wait for device to be idle before destroying resources
            self.device.device_wait_idle().unwrap();    

            //swapchain
            self.destroy_swapchain();

            self.data.command_pools.iter()
                .for_each(|p| self.device.destroy_command_pool(*p, None));

            //images
            self.device.destroy_sampler(self.data.texture_sampler, None);
            self.device.destroy_image_view(self.data.texture_image_view, None);
            self.device.destroy_image(self.data.texture_image, None);
            self.device.free_memory(self.data.texture_image_memory, None);

            //descriptor
            self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);

            //buffers
            self.device.destroy_buffer(self.data.index_buffer, None);
            self.device.free_memory(self.data.index_buffer_memory, None);
            self.device.destroy_buffer(self.data.vertex_buffer, None);
            self.device.free_memory(self.data.vertex_buffer_memory, None);

            //sync objects
            self.data.in_flight_fences.iter()
                .for_each(|f| self.device.destroy_fence(*f, None));
            self.data.render_finished_semaphores.iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
            self.data.image_available_semaphores.iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));


            self.device.destroy_command_pool(self.data.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(self.data.surface, None);

            if VALIDATION_ENABLED {
                self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }   
    
    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device, &mut self.data)?;
        create_descriptor_sets(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;
        Ok(())
    }

    unsafe fn destroy_swapchain(&mut self) {
        //multisampling buffer
        self.device.destroy_image_view(self.data.color_image_view, None);
        self.device.free_memory(self.data.color_image_memory, None);
        self.device.destroy_image(self.data.color_image, None);

        //depth buffer
        self.device.destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);

        //descriptor
        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        
        //uniform buffer
        self.data.uniform_buffers.iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data.uniform_buffers_memory.iter()
            .for_each(|m| self.device.free_memory(*m, None));

        //framebuffers
        self.data.framebuffers.iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));

        //pipeline
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);

        //swapchain
        self.data.swapchain_image_views.iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
    
    pub fn must_resize(&mut self) {
        self.resized = true;
    } 
    
}
