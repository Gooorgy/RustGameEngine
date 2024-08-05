use std::{ffi::CStr, os::raw::c_void, ptr};

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const ash::vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> ash::vk::Bool32 {
    let severity = match message_severity {
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    ash::vk::FALSE
}

pub fn populate_debug_messenger_create_info() -> ash::vk::DebugUtilsMessengerCreateInfoEXT<'static>
{
    ash::vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: ash::vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: ash::vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
        ..Default::default()
    }
}

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enabled: false,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub struct ValidationInfo {
    pub is_enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}
