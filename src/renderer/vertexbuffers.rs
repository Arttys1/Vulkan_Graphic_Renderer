use std::{
    mem::size_of,
    ptr::copy_nonoverlapping as memcpy,
    sync::Arc,
};
use vulkanalia::{
    prelude::v1_0::*
};
use anyhow::{Result, anyhow};
use crate::renderer::{
    vertex::*,
    buffers_tools::*,
};

#[derive(Debug, Clone)]
pub struct VertexBuffer {
    device: Arc<Device>,
    is_allocated: bool,

//vertex buffer
    indices_size: usize,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

//index buffer
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
}

impl VertexBuffer {
    pub fn empty(device: Arc<Device>) -> Result<Self> {
        Ok(
        VertexBuffer {
            device,
            is_allocated: false,
            indices_size: 0,
            // vertices: Vec::default(),
            // indices: Vec::default(),
            vertex_buffer: vk::Buffer::default(),
            vertex_buffer_memory: vk::DeviceMemory::default(),
            index_buffer: vk::Buffer::default(),
            index_buffer_memory: vk::DeviceMemory::default(),
        })
    } 

    pub fn allocate(&mut self, device: Arc<Device>, instance: &Instance,
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Result<()>
    {
        if vertices.is_empty() || indices.is_empty() {
            return Err(anyhow!("vertices or indices can't be empty"));
        }
        if self.is_allocated {
            self.clean();
        }

        unsafe {
            let (vertex_buffer, vertex_buffer_memory) = load_vertex_buffer(instance, &device, physical_device, command_pool, graphics_queue, &vertices)?;
            let (index_buffer, index_buffer_memory) = load_index_buffer(instance, &device, physical_device, command_pool, graphics_queue, &indices)?;
            self.device = device;
            // self.vertices = vertices;
            // self.indices = indices;
            self.indices_size = indices.len();
            self.vertex_buffer = vertex_buffer;
            self.vertex_buffer_memory = vertex_buffer_memory;
            self.index_buffer = index_buffer;
            self.index_buffer_memory = index_buffer_memory;         
            self.is_allocated = true;     
            Ok(())
        }
    }

    pub fn new(device: Arc<Device>, instance: &Instance,
        physical_device: vk::PhysicalDevice, command_pool: vk::CommandPool, 
        graphics_queue: vk::Queue, vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Result<Self>
    {
        let mut buffer = VertexBuffer::empty(device.clone())?;
        buffer.allocate(device.clone(), instance, physical_device, command_pool, graphics_queue, vertices, indices)?;
        Ok(buffer)
    }

    pub fn clean(&mut self) {
        if self.is_allocated {
            unsafe {
                self.device.destroy_buffer(self.index_buffer, None);
                self.device.free_memory(self.index_buffer_memory, None);
                self.device.destroy_buffer(self.vertex_buffer, None);
                self.device.free_memory(self.vertex_buffer_memory, None);
            }
            self.is_allocated = false;
        }
    }

    pub fn vertex_buffer(&self) -> vk::Buffer { self.vertex_buffer }
    pub fn index_buffer(&self) -> vk::Buffer { self.index_buffer }
    pub fn indices_len(&self) -> usize { self.indices_size }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        self.clean();
    }
}


//================================================
// Vertex buffer
//================================================

pub unsafe fn load_vertex_buffer(
    instance: &Instance, 
    device: &Device, 
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool, 
    graphics_queue: vk::Queue,
    vertices: &Vec<Vertex>)
-> Result<(vk::Buffer, vk::DeviceMemory)>
{
    // Create (staging)
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    // Copy (staging)
    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.unmap_memory(staging_buffer_memory);

    // Create (vertex)
    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // Copy (vertex)
    copy_buffer(device, command_pool, graphics_queue, staging_buffer, vertex_buffer, size)?;

    // Cleanup
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((vertex_buffer, vertex_buffer_memory))
}

//================================================
// Index buffer
//================================================

pub unsafe fn load_index_buffer(
    instance: &Instance,
    device: &Device, 
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool, 
    graphics_queue: vk::Queue,
    indices: &Vec<u32>)
-> Result<(vk::Buffer, vk::DeviceMemory)>
{
    let size = (size_of::<u32>() * indices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(indices.as_ptr(), memory.cast(), indices.len());

    device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(device, command_pool, graphics_queue, staging_buffer, index_buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((index_buffer, index_buffer_memory))
}