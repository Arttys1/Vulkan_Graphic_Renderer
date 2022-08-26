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
    },
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    proj: glm::Mat4,
}

impl UniformBufferObject {
    pub fn construct(proj: glm::Mat4) -> Self {
        Self { proj }
    }
    pub fn identity() -> Self {
        Self { proj: glm::identity() }
    }
    pub fn proj(&self) -> glm::Mat4 { self.proj }
    pub fn set_proj(&mut self, proj: glm::Mat4) { self.proj = proj; }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PushConstantObject {
    view: glm::Mat4,
    model: glm::Mat4,
}

impl PushConstantObject {
    pub fn construct(view: glm::Mat4, model: glm::Mat4) -> Self {
        Self { view, model }
    }
    pub fn identity() -> Self {
        Self { view: glm::identity(), model: glm::identity() }
    }
    pub fn view(&self) -> glm::Mat4 { self.view }
    pub fn model(&self) -> glm::Mat4 { self.model }
    pub fn set_view(&mut self, view: glm::Mat4) { self.view = view; }
    pub fn set_model(&mut self, model: glm::Mat4) { self.model = model; }
}

#[derive(Debug, Clone)]
pub struct UniformBuffer {
    device: Arc<Device>,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    fn_update_ubo: fn(u32, u32) -> UniformBufferObject,
    fn_update_push_constant: fn(usize, f32) -> PushConstantObject,
    is_allocated: bool,
}

impl UniformBuffer {
    pub fn new(device: Arc<Device>, instance: &Instance, physical_device: vk::PhysicalDevice, swapchain_images: &Vec<vk::Image>) -> Result<Self> {
        unsafe {
            let (uniform_buffers, 
                uniform_buffers_memory,
            ) = create_uniform_buffers(instance, &device, physical_device, swapchain_images)?;
            Ok(UniformBuffer {
                device,
                uniform_buffers,
                uniform_buffers_memory,
                fn_update_ubo: |_, _| -> UniformBufferObject { UniformBufferObject::identity() },
                fn_update_push_constant: |_, _| -> PushConstantObject { PushConstantObject::identity() },
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

    pub fn set_fn_update_ubo(&mut self, f: fn(u32, u32) -> UniformBufferObject) {
        self.fn_update_ubo = f;
    }
    pub fn set_fn_update_push_constant(&mut self, f: fn(usize, f32) -> PushConstantObject) {
        self.fn_update_push_constant = f;
    }

    pub unsafe fn update_uniform_buffer(&mut self, device: &Device, swapchain_extent: vk::Extent2D, image_index: usize) -> Result<()> {
        let fn_update = self.fn_update_ubo;
        let ubo = fn_update(swapchain_extent.width, swapchain_extent.height);
    
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

    pub unsafe fn update_push_constant(&self, model_index: usize, elapsed_time: f32) -> PushConstantObject {
        let push_constant_func = self.fn_update_push_constant;
        let push_constant_object = push_constant_func(model_index, elapsed_time);
        push_constant_object
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