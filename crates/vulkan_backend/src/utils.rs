use crate::device::DeviceInfo;
use crate::scene::ImageResource;
use ash::vk::{CompareOp, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, Sampler};
use ash::{vk, Instance};
use std::mem;
use std::path::Path;

pub fn find_memory_type(
    type_filter: u32,
    properties: MemoryPropertyFlags,
    memory_properties: PhysicalDeviceMemoryProperties,
) -> u32 {
    for (i, memory_type) in memory_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(properties) {
            return i as u32;
        }
    }

    panic!()
}

pub fn get_buffer_alignment<T>(device_info: &DeviceInfo) -> u64 {
    let mut dynamic_alignment = mem::size_of::<T>() as u64;
    let min_ubo_alignment = device_info.min_ubo_alignment;

    if min_ubo_alignment > 0 {
        dynamic_alignment = (dynamic_alignment + min_ubo_alignment - 1) & !(min_ubo_alignment - 1);
    }

    dynamic_alignment
}

pub fn create_texture_sampler(
    device_info: &DeviceInfo,
    instance: &Instance,
    shadow: bool,
) -> Sampler {
    let device_properties =
        unsafe { instance.get_physical_device_properties(device_info._physical_device) };

    let compare_op = if shadow {
        CompareOp::LESS
    } else {
        CompareOp::ALWAYS
    };

    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(true)
        .max_anisotropy(device_properties.limits.max_sampler_anisotropy)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(shadow)
        .compare_op(compare_op)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);

    unsafe {
        device_info
            .logical_device
            .create_sampler(&sampler_info, None)
            .expect("failed to create sampler")
    }
}

pub fn load_texture<P>(path: P) -> ImageResource
where
    P: AsRef<Path>,
{
    let dyn_image = image::open(path).unwrap();
    let image_width = dyn_image.width();
    let image_height = dyn_image.height();

    let image_data = match &dyn_image {
        image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => {
            dyn_image.to_rgba8().into_raw()
        }
        _ => vec![],
    };

    ImageResource {
        image_data,
        width: image_width,
        height: image_height,
    }
}
