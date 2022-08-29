use {
    vulkanalia::{
        prelude::v1_0::*,
        window as vk_window,
        vk::{KhrSwapchainExtension, KhrSurfaceExtension, ExtDebugUtilsExtension},
    },
    std::sync::Arc,
    winit::window::Window,
    super::{
        instance::{create_instance, VALIDATION_ENABLED},
        queue_family::{pick_physical_device, create_logical_device},
        swapchain::{create_swapchain, create_swapchain_image_views}, 
        pipeline::{create_render_pass, create_pipeline}, 
        descriptor::create_descriptor_set_layout, 
        commandbuffers::create_command_pools, 
        image::create_color_objects, 
        depthbuffers::create_depth_objects, 
        framebuffers::create_framebuffers, 
        sync::create_sync_objects,
        vulkan_model::VulkanModel,
    },
    anyhow::Result,
};
/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug)]
pub struct AppData {
    instance: Instance,
    device: Arc<Device>,
    surface: vk::SurfaceKHR,
    messenger: vk::DebugUtilsMessengerEXT,

//physical device
    msaa_samples: vk::SampleCountFlags,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

//swapchain
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,

//pipeline
    pipeline: vk::Pipeline,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    
//framebuffers
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    command_pools: Vec<vk::CommandPool>,

//sync
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,

//depth buffer
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,

//multisampling
    color_image: vk::Image,
    color_image_memory: vk::DeviceMemory,
    color_image_view: vk::ImageView,

    models: Vec<VulkanModel>,
    is_allocated: bool,
}

impl AppData {
    pub fn new(window: &Window, entry: &Entry) -> Result<Self> {
        unsafe {
            let (instance, messenger) = create_instance(window, entry)?;
            let surface = vk_window::create_surface(&instance, window)?;

            let (physical_device, msaa_samples) = pick_physical_device(&instance, surface)?;
            let (device_,
                graphics_queue,
                present_queue) = create_logical_device(&instance, surface, physical_device)?;
            let device = Arc::new(device_);

            let (swapchain,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
            ) = create_swapchain(window, &instance, &device, surface, physical_device)?;
            let swapchain_image_views = create_swapchain_image_views(&device, &swapchain_images, swapchain_format)?;

            let render_pass = create_render_pass(&instance, &device, physical_device, swapchain_format, msaa_samples)?;
            let descriptor_set_layout = create_descriptor_set_layout(&device)?;
                    
            let ( pipeline, 
                pipeline_layout
            ) = create_pipeline(&device, swapchain_extent, msaa_samples, descriptor_set_layout, render_pass)?;
            
            let (command_pool,
                command_pools
            ) = create_command_pools(&instance, &device, &swapchain_images, surface, physical_device)?;

            let (color_image, 
                color_image_memory, 
                color_image_view,
            ) = create_color_objects(&instance, &device, physical_device, swapchain_extent, msaa_samples, swapchain_format)?;
            
            let (depth_image, 
                depth_image_memory, 
                depth_image_view,
            ) = create_depth_objects(&instance, &device, physical_device, swapchain_extent, msaa_samples)?;
            
            let framebuffers = create_framebuffers(&device, &swapchain_image_views, 
                swapchain_extent, render_pass, 
                depth_image_view, color_image_view)?;

            let command_buffers = vec![vk::CommandBuffer::null(); framebuffers.len()];
            
            let (in_flight_fences,
                render_finished_semaphores,
                image_available_semaphores,
                images_in_flight,
                ) = create_sync_objects(&device, &swapchain_images)?;

            Ok(AppData {
                instance,
                device,
                surface,
                messenger,
                msaa_samples,
                physical_device,
                graphics_queue,
                present_queue,
                swapchain,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
                swapchain_image_views,
                pipeline,
                render_pass,
                descriptor_set_layout,
                pipeline_layout,
                framebuffers,
                command_pool,
                command_buffers,
                command_pools,
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
                images_in_flight,
                depth_image,
                depth_image_memory,
                depth_image_view,
                color_image,
                color_image_memory,
                color_image_view,
                models: vec![],
                is_allocated: true,
            })
        }        
    }

    /// Destroys our Vulkan app.
    pub fn clean(&mut self) {
        if self.is_allocated {
            unsafe {
                //wait for device to be idle before destroying resources
                self.device.device_wait_idle().unwrap();    

                //swapchain
                self.destroy_swapchain();

                for i in 0..self.models.len() {
                    let model = &mut self.models[i];
                    model.clean();
                }

                self.command_pools.iter()
                    .for_each(|p| self.device.destroy_command_pool(*p, None));

                //sync objects
                self.in_flight_fences.iter()
                    .for_each(|f| self.device.destroy_fence(*f, None));
                self.render_finished_semaphores.iter()
                    .for_each(|s| self.device.destroy_semaphore(*s, None));
                self.image_available_semaphores.iter()
                    .for_each(|s| self.device.destroy_semaphore(*s, None));

                self.device.destroy_command_pool(self.command_pool, None);                
                self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
                self.device.destroy_device(None);
                self.instance.destroy_surface_khr(self.surface, None);

                if VALIDATION_ENABLED {
                    self.instance.destroy_debug_utils_messenger_ext(self.messenger, None);
                }

                self.instance.destroy_instance(None);
                self.is_allocated = false;
            }
        }
    }

    unsafe fn destroy_swapchain(&mut self) {
        //multisampling buffer
        self.device.destroy_image_view(self.color_image_view, None);
        self.device.free_memory(self.color_image_memory, None);
        self.device.destroy_image(self.color_image, None);

        //depth buffer
        self.device.destroy_image_view(self.depth_image_view, None);
        self.device.free_memory(self.depth_image_memory, None);
        self.device.destroy_image(self.depth_image, None);

        //framebuffers
        self.framebuffers.iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));

        //pipeline
        self.device.destroy_pipeline(self.pipeline, None);
        self.device.destroy_pipeline_layout(self.pipeline_layout, None);
        self.device.destroy_render_pass(self.render_pass, None);

        //swapchain
        self.swapchain_image_views.iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));

        self.device.destroy_swapchain_khr(self.swapchain, None);
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        let instance = &self.instance;
        let device = &self.device;
        let surface = &self.surface;
        let physical_device = &self.physical_device;
        let msaa_samples = &self.msaa_samples;

        let (swapchain,
            swapchain_format,
            swapchain_extent,
            swapchain_images,
        ) = create_swapchain(window, instance, &device, *surface, *physical_device)?;
        let swapchain_image_views = create_swapchain_image_views(&device, &swapchain_images, swapchain_format)?;

        let render_pass = create_render_pass(&instance, &device, *physical_device, swapchain_format, *msaa_samples)?;
                
        let ( pipeline, 
            pipeline_layout
        ) = create_pipeline(&device, swapchain_extent, *msaa_samples, self.descriptor_set_layout, render_pass)?;

        let (color_image, 
            color_image_memory, 
            color_image_view,
        ) = create_color_objects(instance, &device, *physical_device, swapchain_extent, *msaa_samples, swapchain_format)?;
        
        let (depth_image, 
            depth_image_memory, 
            depth_image_view,
        ) = create_depth_objects(instance, &device,* physical_device, swapchain_extent, *msaa_samples)?;
        
        let framebuffers = create_framebuffers(&device, &swapchain_image_views, 
            swapchain_extent, render_pass, 
            depth_image_view, color_image_view)?;

        self.swapchain = swapchain;
        self.swapchain_format = swapchain_format;
        self.swapchain_extent = swapchain_extent;
        self.swapchain_images = swapchain_images;
        self.swapchain_image_views = swapchain_image_views;
        self.pipeline = pipeline;
        self.render_pass = render_pass;
        self.pipeline_layout = pipeline_layout;
        self.framebuffers = framebuffers;
        self.color_image = color_image;
        self.color_image_memory = color_image_memory;
        self.color_image_view = color_image_view;
        self.depth_image = depth_image;
        self.depth_image_memory = depth_image_memory;
        self.depth_image_view = depth_image_view;
        self.command_buffers = vec![vk::CommandBuffer::null(); self.framebuffers.len()];

        for i in 0..self.models.len() {
            let model = &mut self.models[i];
            model.reload_swapchain(
                &self.instance,
                self.physical_device,
                &self.swapchain_images, 
                self.descriptor_set_layout)?;
        }

        Ok(())
    }

    pub fn push_model(&mut self, model: VulkanModel) { self.models.push(model); }
    pub fn models(&self) -> &[VulkanModel] { self.models.as_ref() }
    pub fn at_model(&self, index: usize) -> &VulkanModel { &self.models[index] }

    //getters
    pub fn device(&self) -> Arc<Device> { self.device.clone() }
    pub fn instance(&self) -> &Instance { &self.instance }
    pub fn swapchain_extent(&self) -> vk::Extent2D { self.swapchain_extent }
    pub fn render_pass(&self) -> vk::RenderPass { self.render_pass } 
    pub fn pipeline_layout(&self) -> vk::PipelineLayout { self.pipeline_layout } 
    pub fn command_pools(&self) -> &[vk::CommandPool] { self.command_pools.as_ref() }
    pub fn command_buffers(&self) -> &[vk::CommandBuffer] { self.command_buffers.as_ref() }
    pub fn pipeline(&self) -> vk::Pipeline { self.pipeline }
    pub fn framebuffers(&self) -> &[vk::Framebuffer] { self.framebuffers.as_ref() }
    pub fn graphics_queue(&self) -> vk::Queue { self.graphics_queue }
    pub fn swapchain(&self) -> vk::SwapchainKHR { self.swapchain }
    pub fn image_available_semaphores(&self) -> &[vk::Semaphore] { self.image_available_semaphores.as_ref() }
    pub fn render_finished_semaphores(&self) -> &[vk::Semaphore] { self.render_finished_semaphores.as_ref() }
    pub fn in_flight_fences(&self) -> &[vk::Fence] { self.in_flight_fences.as_ref() }
    pub fn images_in_flight(&self) -> &[vk::Fence] { self.images_in_flight.as_ref() }
    pub fn present_queue(&self) -> vk::Queue { self.present_queue } 
    pub fn physical_device(&self) -> vk::PhysicalDevice { self.physical_device }
    pub fn command_pool(&self) -> vk::CommandPool { self.command_pool }
    pub fn swapchain_images(&self) -> &Vec<vk::Image> { &self.swapchain_images }
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout { self.descriptor_set_layout }
    pub fn images_in_flight_mut(&mut self) -> &mut Vec<vk::Fence> { &mut self.images_in_flight }
    pub fn command_buffers_mut(&mut self) -> &mut Vec<vk::CommandBuffer> { &mut self.command_buffers }
    
}

impl Drop for AppData {
    fn drop(&mut self) {
        self.clean();
    }
}