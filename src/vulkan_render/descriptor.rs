use crate::vulkan_render::buffer::AllocatedBuffer;
use crate::vulkan_render::device::DeviceInfo;
use crate::vulkan_render::frame_manager::ShadowCascade;
use crate::vulkan_render::structs::{CameraMvpUbo, CascadeShadowUbo, LightingUbo};
use ash::vk::{
    DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet,
    DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType, ImageView,
};
use ash::{vk, Device};
use std::{mem, slice};

/// Uniform buffer count: 1 for camera, 1 for lighting
const GLOBAL_UNIFORM_BUFFER_COUNT: usize = 20;

/// Dynamic buffer count: 1 for model matrix
const GLOBAL_DYNAMIC_UNIFORM_BUFFER_COUNT: usize = 20;

/// Global image sampler count: 4 for albedo, normal, depth, shadow-map
const GLOBAL_IMAGE_SAMPLER_COUNT: usize = 20;

pub struct DescriptorManager {
    pub global_pool: DescriptorPool,
    pub global_gbuffer_layout: DescriptorSetLayout,
    pub global_lighting_layout: DescriptorSetLayout,
    pub global_shadow_map_layout: DescriptorSetLayout,
}

impl DescriptorManager {
    pub fn new(device: &Device, max_frames: usize) -> Self {
        let global_pool = Self::create_global_pool(device, max_frames);
        let global_gbuffer_layout = Self::create_global_gbuffer_layout(device);
        let global_lighting_layout = Self::create_global_lighting_layout(device);
        let global_shadow_map_layout = Self::create_global_shadow_map_layout(device);

        Self {
            global_pool,
            global_gbuffer_layout,
            global_lighting_layout,
            global_shadow_map_layout,
        }
    }

    pub fn create_gbuffer_descriptor_set(&self, device: &Device) -> DescriptorSet {
        let binding = [self.global_gbuffer_layout];
        let allocate_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.global_pool)
            .set_layouts(&binding);

        unsafe {
            device
                .allocate_descriptor_sets(&allocate_info)
                .expect("failed to create descriptor set layout")[0]
        }
    }

    pub fn create_lighting_descriptor_set(&self, device: &Device) -> DescriptorSet {
        let binding = [self.global_lighting_layout];
        let allocate_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.global_pool)
            .set_layouts(&binding);

        unsafe {
            device
                .allocate_descriptor_sets(&allocate_info)
                .expect("failed to create descriptor set layout")[0]
        }
    }

    pub fn update_gbuffer_descriptor_set(
        &self,
        device_info: &DeviceInfo,
        camera_mvp_buffer: &AllocatedBuffer,
        dynamic_model_buffer: &AllocatedBuffer,
        dynamic_alignment: u64,
        texture_image_view: &ImageView,
        texture_sampler: &vk::Sampler,
        descriptor_set: DescriptorSet,
    ) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(camera_mvp_buffer.buffer)
            .offset(0)
            .range(mem::size_of::<CameraMvpUbo>() as u64);

        let dynamic_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(dynamic_model_buffer.buffer)
            .offset(0)
            .range(dynamic_alignment);

        let image_info = [vk::DescriptorImageInfo::default()
            .image_view(*texture_image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(*texture_sampler)];

        let mut write_descriptor_sets = vec![];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&buffer_info)),
        );

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&dynamic_buffer_info)),
        );

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(image_info.as_slice()),
        );

        println!("Write descriptor sets");
        unsafe {
            device_info
                .logical_device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
        }
    }

    pub fn update_dynamic_buffer_descriptor_sets(
        &self,
        device_info: &DeviceInfo,
        dynamic_model_buffer: &AllocatedBuffer,
        descriptor_set: DescriptorSet,
        alignment: u64,
    ) {
        let dynamic_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(dynamic_model_buffer.buffer)
            .offset(0)
            .range(alignment);

        let mut write_descriptor_sets = vec![];
        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&dynamic_buffer_info)),
        );

        unsafe {
            device_info
                .logical_device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
        }
    }

    pub fn create_shadow_map_descriptor_set(&self, device: &Device) -> DescriptorSet {
        let binding = [self.global_shadow_map_layout];
        let allocate_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.global_pool)
            .set_layouts(&binding);

        unsafe {
            device
                .allocate_descriptor_sets(&allocate_info)
                .expect("failed to create descriptor set layout")[0]
        }
    }

    pub fn update_shadow_map_descriptor_set(
        &self,
        device_info: &DeviceInfo,
        shadow_map_buffer: &AllocatedBuffer,
        descriptor_set: DescriptorSet,
    ) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(shadow_map_buffer.buffer)
            .offset(0)
            .range(mem::size_of::<CascadeShadowUbo>() as u64 * 3);

        let mut write_descriptor_sets = vec![];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&buffer_info)),
        );

        unsafe {
            device_info
                .logical_device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
        }
    }

    pub fn update_lighting_descriptor_set(
        &self,
        device_info: &DeviceInfo,
        lighting_buffer: &AllocatedBuffer,
        albedo_image_view: &ImageView,
        albedo_sampler: &vk::Sampler,
        normal_image_view: &ImageView,
        normal_sampler: &vk::Sampler,
        depth_image_view: &ImageView,
        depth_sampler: &vk::Sampler,
        descriptor_set: DescriptorSet,
        cascade: &Vec<ShadowCascade>,
        shadow_map_buffer: &AllocatedBuffer,
        camera_mvp_buffer: &AllocatedBuffer,
        pos_image_view: &ImageView,
        pos_sampler: &vk::Sampler,
    ) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(lighting_buffer.buffer)
            .offset(0)
            .range(mem::size_of::<LightingUbo>() as u64);

        let albedo_info = [vk::DescriptorImageInfo::default()
            .image_view(*albedo_image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(*albedo_sampler)];

        let normal_info = [vk::DescriptorImageInfo::default()
            .image_view(*normal_image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(*normal_sampler)];

        let depth_info = [vk::DescriptorImageInfo::default()
            .image_view(*depth_image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(*depth_sampler)];

        let mut write_descriptor_sets = vec![];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&buffer_info)),
        );

        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(shadow_map_buffer.buffer)
            .offset(0)
            .range(mem::size_of::<CascadeShadowUbo>() as u64 * (cascade.len()) as u64);

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(7)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&buffer_info)),
        );

        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(camera_mvp_buffer.buffer)
            .offset(0)
            .range(mem::size_of::<CameraMvpUbo>() as u64);

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(8)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(slice::from_ref(&buffer_info)),
        );

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(albedo_info.as_slice()),
        );

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(normal_info.as_slice()),
        );

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(3)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(depth_info.as_slice()),
        );

        let depth_info = [vk::DescriptorImageInfo::default()
            .image_view(cascade[0].shadow_depth_image.image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(cascade[0].cascade_sampler)];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(4)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(depth_info.as_slice()),
        );

        let depth_info = [vk::DescriptorImageInfo::default()
            .image_view(cascade[1].shadow_depth_image.image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(cascade[1].cascade_sampler)];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(5)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(depth_info.as_slice()),
        );

        let depth_info = [vk::DescriptorImageInfo::default()
            .image_view(cascade[2].shadow_depth_image.image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(cascade[2].cascade_sampler)];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(6)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(depth_info.as_slice()),
        );

        let pos_info = [vk::DescriptorImageInfo::default()
            .image_view(*pos_image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(*pos_sampler)];

        write_descriptor_sets.push(
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(9)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(pos_info.as_slice()),
        );

        unsafe {
            device_info
                .logical_device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
        }
    }

    fn create_global_pool(device: &Device, max_frames: usize) -> DescriptorPool {
        let pool_sizes = [
            DescriptorPoolSize::default()
                .descriptor_count((max_frames * GLOBAL_UNIFORM_BUFFER_COUNT) as u32)
                .ty(DescriptorType::UNIFORM_BUFFER),
            DescriptorPoolSize::default()
                .descriptor_count((max_frames * GLOBAL_DYNAMIC_UNIFORM_BUFFER_COUNT) as u32)
                .ty(DescriptorType::UNIFORM_BUFFER_DYNAMIC),
            DescriptorPoolSize::default()
                .descriptor_count((max_frames * GLOBAL_IMAGE_SAMPLER_COUNT) as u32)
                .ty(DescriptorType::COMBINED_IMAGE_SAMPLER),
        ];

        let create_info = DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(max_frames as u32 * 4);

        unsafe {
            device
                .create_descriptor_pool(&create_info, None)
                .expect("Failed to create global descriptor pool")
        }
    }

    fn create_global_lighting_layout(device: &Device) -> DescriptorSetLayout {
        let bindings = [
            // Lighting Data
            DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            // Albedo Texture
            DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            // Normal Texture
            DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            // Depth Texture
            DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            // Shadow map
            DescriptorSetLayoutBinding::default()
                .binding(4)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            DescriptorSetLayoutBinding::default()
                .binding(5)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            DescriptorSetLayoutBinding::default()
                .binding(6)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            DescriptorSetLayoutBinding::default()
                .binding(7)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            DescriptorSetLayoutBinding::default()
                .binding(8)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            DescriptorSetLayoutBinding::default()
                .binding(9)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];

        let create_info = DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::empty());

        unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .expect("Failed to create global lighting descriptor set")
        }
    }

    fn create_global_gbuffer_layout(device: &Device) -> DescriptorSetLayout {
        let bindings = [
            // Camera Data
            DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            // Model offset Texture
            DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            // TODO: Create Material system
            // Texture
            DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];

        let create_info = DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::empty());

        unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .expect("Failed to create global gBuffer descriptor set layout")
        }
    }

    fn create_global_shadow_map_layout(device: &Device) -> DescriptorSetLayout {
        let bindings = [DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];

        let create_info = DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::empty());

        unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .expect("Failed to create global shadow map descriptor set layout")
        }
    }
}
