use {
    std::{
        mem::size_of,
        ptr::copy_nonoverlapping as memcpy,
        sync::Arc,
    },
    vulkanalia::prelude::v1_0::*,
    anyhow::Result,
    nalgebra_glm as glm,
    super::{
        buffers_tools::create_buffer,
        renderer::Renderer,
    },
};

#[derive(Debug, Clone)]
pub struct UniformBuffer {
    device: Arc<Device>,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    is_allocated: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    view: glm::Mat4,
    proj: glm::Mat4,
}

impl UniformBufferObject {
    pub fn construct(view: glm::Mat4, proj: glm::Mat4) -> Self {
        Self { view, proj }
    }
}

impl UniformBuffer {
    pub fn new(renderer: &Renderer) -> Result<Self> {
        let device = renderer.get_device();
        let data = renderer.get_appdata();
        unsafe {
            let (uniform_buffers, uniform_buffers_memory) = create_uniform_buffers(renderer.get_instance(), &device, data.physical_device(), &data.swapchain_images())?;
            Ok(UniformBuffer {
                device,
                uniform_buffers,
                uniform_buffers_memory,
                is_allocated: true,
            })
        }

    }

    pub fn clean(&mut self) {
        if self.is_allocated {
            unsafe {
            self.uniform_buffers.iter()
                .for_each(|b| self.device.destroy_buffer(*b, None));
            self.uniform_buffers_memory.iter()
                .for_each(|m| self.device.free_memory(*m, None));
            }
            self.is_allocated = false;
        }
    }

    pub fn reload_swapchain_models(&mut self, instance: &Instance, physical_device: vk::PhysicalDevice, swapchain_images: &Vec<vk::Image>) -> Result<()> {
        self.clean();
        unsafe {
            let (uniform_buffers, uniform_buffers_memory) = create_uniform_buffers(instance, &self.device, physical_device, swapchain_images)?;
            self.uniform_buffers = uniform_buffers;
            self.uniform_buffers_memory = uniform_buffers_memory;
            self.is_allocated = true;
        }
        Ok(())
    }

    pub unsafe fn update_uniform_buffer(&mut self, device: &Device, swapchain_extent: vk::Extent2D, image_index: usize) -> Result<()> {
        let view = glm::look_at(
            &glm::vec3(6.0, 0.0, 2.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 0.0, 1.0),
        );
    
        let mut proj = glm::perspective_rh_zo(
            swapchain_extent.width as f32 / swapchain_extent.height as f32,
            glm::radians(&glm::vec1(45.0))[0],
            0.1,
            10.0,
        );        
        proj[(1, 1)] *= -1.0;
        
        let ubo = UniformBufferObject::construct(view, proj);
    
        let memory = device.map_memory(
            self.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;
        
        memcpy(&ubo, memory.cast(), 1);
        
        device.unmap_memory(self.uniform_buffers_memory[image_index]);    
    
        Ok(())
    }

    pub(crate) fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    pub fn uniform_buffers(&self) -> &Vec<vk::Buffer> {
       &self.uniform_buffers 
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        self.clean();
    }
}

//================================================
// uniform buffers
//================================================

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    swapchain_images: &Vec<vk::Image>)
-> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
    let mut uniform_buffers : Vec<vk::Buffer> = Vec::default(); 
    let mut uniform_buffers_memory : Vec<vk::DeviceMemory> = Vec::default(); 

    for _ in 0..swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        uniform_buffers.push(uniform_buffer);
        uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok((uniform_buffers, uniform_buffers_memory))
}