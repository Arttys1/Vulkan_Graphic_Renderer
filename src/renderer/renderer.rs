use {
    std::{time::Instant, sync::Arc},
    vulkanalia::{
        loader::{ LibloadingLoader, LIBRARY },
        prelude::v1_0::*,
        vk::{
            KhrSwapchainExtension,
        },
    },
    crate::tools::{texture::Texture, model::Model},
    nalgebra_glm as glm,
    winit::window::Window,
    anyhow::{anyhow, Result},
    super::{
        appdata::*,
        commandbuffers::*, 
        vulkan_model::VulkanModel,
        vertex::Vertex, 
        vertexbuffers::VertexBuffer, 
        vulkan_texture::VulkanTexture, 
        uniformbuffers::{UniformBuffer, UniformBufferObject, PushConstantObject}, 
        descriptor::Descriptor,
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
            self.data.update_models(image_index)?;

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

    pub fn create_model(&mut self, model: Arc<Model>, texture: Arc<Texture>) -> Result<()> {
        let model = VulkanModel::read(
            self.data.device(),
            self.data.instance(),
            self.data.physical_device(),
            self.data.command_pool(),
            self.data.graphics_queue(),
            self.data.swapchain_images(),
            self.data.descriptor_set_layout(),
            model, texture)?;
        self.data.push_model(model);
        Ok(())
    }

    pub fn construct_model(&mut self, vertices: &Vec<Vertex>, indices: &Vec<u32>, texture: Arc<Texture>) -> Result<()> {
        let device = self.data.device();
        let instance = self.data.instance();
        let physical_device = self.data.physical_device();
        let command_pool = self.data.command_pool();
        let graphics_queue = self.data.graphics_queue();
        let swapchain_images = self.data.swapchain_images();
        let descriptor_set_layout = self.data.descriptor_set_layout();

        let buffer = VertexBuffer::new(device.clone(), instance, physical_device, command_pool, graphics_queue, vertices, indices)?;
        let texture = VulkanTexture::new(device.clone(), instance,physical_device,command_pool,graphics_queue, texture)?;
        let mut uniform_buffer = UniformBuffer::new(device.clone(), instance,physical_device,swapchain_images)?;
        uniform_buffer.set_fn_update_ubo(Renderer::update_ubo);
        uniform_buffer.set_fn_update_push_constant(Renderer::update_push_constant);
        let descriptor = Descriptor::new(device.clone(),             
            swapchain_images,
            descriptor_set_layout, 
            &uniform_buffer, 
            &texture)?;

        let model = VulkanModel::construct(buffer, texture, uniform_buffer, descriptor)?;    
        self.data.push_model(model);
        Ok(())
    }

    pub(crate) fn update_ubo(swapchain_width: u32, swapchain_height: u32) -> UniformBufferObject {        
            let mut proj = glm::perspective_rh_zo(
                swapchain_width as f32 / swapchain_height as f32,
                glm::radians(&glm::vec1(45.0))[0],
                0.1,
                10.0,
            );        
            proj[(1, 1)] *= -1.0;
            UniformBufferObject::construct(proj)
    }
    pub(crate) fn update_push_constant(model_index: usize, elapsed_time: f32) -> PushConstantObject {
        let y = (((model_index % 2) as f32) * 2.5) - 1.25;
        let z = (((model_index / 2) as f32) * -2.0) + 1.0;

        let model = glm::translate(
            &glm::identity(),
            &glm::vec3(0.0, y, z),
        );    
        let model = glm::rotate(
            &model,
            elapsed_time * glm::radians(&glm::vec1(90.0))[0],
            &glm::vec3(0.0, 0.0, 1.0),
        );
        let view = glm::look_at(
            &glm::vec3(6.0f32, 0.0, 2.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 0.0, 1.0),
        );
        PushConstantObject::construct(view, model)
    }

    pub fn clean(&mut self) {
        self.data.clean();
    }
    
    pub fn add_model(&mut self, model: VulkanModel) {
        self.data.push_model(model);
    }

    pub fn must_resize(&mut self) {
        self.resized = true;
    } 
    
    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn get_instance(&self) -> &Instance {
        &self.data.instance()
    }

    pub fn get_appdata(&self) -> &AppData {
        &self.data
    }

}
