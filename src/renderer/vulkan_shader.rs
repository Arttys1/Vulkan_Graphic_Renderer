use super::descriptor::create_descriptor_set_layout;

use {
    std::sync::Arc,
    anyhow::Result,
    vulkanalia::prelude::v1_0::*,
    super::pipeline::create_pipeline,
};

#[derive(Copy, Clone)]
pub enum ShaderType {
    Textured,
    Untextured,
}

#[derive(Clone, Debug)]
pub struct VulkanShader {
    device: Arc<Device>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    is_allocated: bool,
}

impl VulkanShader {
    pub fn new(device: Arc<Device>, shader_type: ShaderType,
        swapchain_extent: vk::Extent2D, 
        msaa_samples: vk::SampleCountFlags,
        render_pass: vk::RenderPass) -> Result<Self> {
        unsafe {
            let descriptor_set_layout = create_descriptor_set_layout(&device, shader_type)?;
                        
            let ( pipeline, 
                pipeline_layout
            ) = create_pipeline(&device, swapchain_extent, msaa_samples, descriptor_set_layout, render_pass)?;
            Ok(Self {
                device,
                descriptor_set_layout,
                pipeline,
                pipeline_layout,
                is_allocated: true,
            })
        }
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
            ) = create_pipeline(&self.device, swapchain_extent, msaa_samples, self.descriptor_set_layout, render_pass)?;
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