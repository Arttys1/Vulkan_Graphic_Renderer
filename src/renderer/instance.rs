use {
    vulkanalia::{
        prelude::v1_0::*,
        Entry,
        vk::{
            ExtDebugUtilsExtension,
        },
        window as vk_window,
    },
    winit::window::Window,
    anyhow::{Result, anyhow},
    std::collections::HashSet,
    
    log::*,
    std::{ ffi::CStr, os::raw::c_void, },
};

pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

//================================================
// Instance
//================================================

pub unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<(Instance, vk::DebugUtilsMessengerEXT)> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"Mercurian\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let layer_properties = entry.enumerate_instance_layer_properties()?;
    let available_layers = layer_properties.iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();
    
    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }
    
    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };    

    let mut extensions = vk_window::get_required_instance_extensions(window)
    .iter()
    .map(|e| e.as_ptr())
    .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let mut info = vk::InstanceCreateInfo::builder()
    .application_info(&application_info)
    .enabled_layer_names(&layers)
    .enabled_extension_names(&extensions);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }
    
    let instance = entry.create_instance(&info, None)?;

    let mut messenger = vk::DebugUtilsMessengerEXT::default();
    if VALIDATION_ENABLED {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok((instance, messenger))

}

//================================================
// Validation Layer
//================================================

pub extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void) -> vk::Bool32 {

    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}
