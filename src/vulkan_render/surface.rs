use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct SurfaceInfo {
    pub surface_instance: ash::khr::surface::Instance,
    pub surface: ash::vk::SurfaceKHR,
}

impl SurfaceInfo {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> SurfaceInfo {
        let window_handle = window.window_handle().unwrap().as_raw();
        let display_handle = window.display_handle().unwrap().as_raw();
        let surface = unsafe {
            ash_window::create_surface(entry, instance, display_handle, window_handle, None)
                .unwrap()
        };

        let surface_instance = ash::khr::surface::Instance::new(entry, instance);

        Self {
            surface_instance,
            surface,
        }
    }
}
