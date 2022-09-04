use {
    std::{
        sync::Arc,
        collections::HashMap,
        cell::RefCell,
    },
    anyhow::Result,
    vulkanalia::prelude::v1_0::*,
    super::{
        descriptor::create_descriptor_set_layout,
        pipeline::create_pipeline_type,
    },
};

#[derive(Clone)]
pub struct ShaderContainer {
    device: Arc<Device>,
    shaders: HashMap<ShaderType, Arc<RefCell<VulkanShader>>>,
}

impl ShaderContainer {
    pub fn new(device: Arc<Device>) -> Self {
        Self {device, shaders: HashMap::default()}
    }
    pub fn get(&mut self, shader_type: ShaderType,
        swapchain_extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags, 
        render_pass: vk::RenderPass) -> Result<Arc<RefCell<VulkanShader>>>
    {
        if let Some(shader) = self.shaders.get(&shader_type) {
            Ok(shader.clone())
        }
        else {
            let shader = Arc::new(RefCell::new(
                VulkanShader::new(
                    self.device.clone(),
                    shader_type,
                    swapchain_extent,
                    msaa_samples,
                    render_pass,
                )?));
            self.shaders.insert(shader_type, shader.clone());
            Ok(shader.clone())                    
        }
    }

    pub fn reload_swapchain(&mut self, 
        swapchain_extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
        render_pass: vk::RenderPass) -> Result<()> 
    {
        for (_, ptr_shader) in self.shaders.iter() {
            unsafe {
                let mut_ptr_shader = ptr_shader.as_ptr().as_mut();
                if let Some(shader) = mut_ptr_shader {
                    shader.reload_swapchain(swapchain_extent, msaa_samples, render_pass)?;
                }
            }
        }
        Ok(())
    }

    pub fn clean(&mut self) {
        for (_, ptr_shader) in self.shaders.iter() {
            unsafe {
                let mut_ptr_shader = ptr_shader.as_ptr().as_mut();
                if let Some(shader) = mut_ptr_shader {
                    shader.clean();
                }
            }
        }
    } 
}

impl Drop for ShaderContainer {
    fn drop(&mut self) {
        self.clean();
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ShaderType {
    Textured,
    Untextured,
}

#[derive(Clone, Debug)]
pub struct VulkanShader {
    device: Arc<Device>,
    shader_type: ShaderType,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    is_allocated: bool,
}

impl VulkanShader {
    pub fn new(device: Arc<Device>, shader_type: ShaderType,
        swapchain_extent: vk::Extent2D, 
        msaa_samples: vk::SampleCountFlags,
        render_pass: vk::RenderPass) -> Result<Self> 
    {
        let descriptor_set_layout = create_descriptor_set_layout(&device, shader_type)?;
                    
        let ( pipeline, 
            pipeline_layout
        ) = create_pipeline_type(&device, shader_type, swapchain_extent, msaa_samples, descriptor_set_layout, render_pass)?;
        Ok(Self {
            device,
            shader_type,
            descriptor_set_layout,
            pipeline,
            pipeline_layout,
            is_allocated: true,
        })
        
    }

    pub fn clean(&mut self) {
        if self.is_allocated {
            unsafe {
                self.device.destroy_pipeline(self.pipeline, None);
                self.device.destroy_pipeline_layout(self.pipeline_layout, None);
                self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
                self.is_allocated = false;
            }
        }
    }

    pub fn reload_swapchain(&mut self, 
        swapchain_extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
        render_pass: vk::RenderPass) -> Result<()> {
        unsafe {
            if self.is_allocated {                
                    self.device.destroy_pipeline(self.pipeline, None);
                    self.device.destroy_pipeline_layout(self.pipeline_layout, None);                
            }
            let (pipeline, 
                pipeline_layout
            ) = create_pipeline_type(&self.device, self.shader_type, swapchain_extent, msaa_samples, self.descriptor_set_layout, render_pass)?;
            self.pipeline = pipeline;
            self.pipeline_layout = pipeline_layout;
        }
        Ok(())
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for VulkanShader {
    fn drop(&mut self) {
        self.clean();
    }
}