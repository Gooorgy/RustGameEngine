use ash::Device;

/// Trait for GPU resources that can be explicitly freed via a Vulkan device.
/// Implement this on every resource wrapper that owns Vulkan handles.
pub(crate) trait Destroyable {
    fn destroy(&self, device: &Device);
}
