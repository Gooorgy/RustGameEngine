use crate::descriptor::{DescriptorLayoutDesc, DescriptorType, DescriptorWrite, ShaderStage};
use std::slice;

use crate::backend_impl::device::DeviceInfo;
use ash::vk;

pub struct DescriptorPoolChunk {
    pub pool: vk::DescriptorPool,
    pub used: u32,
    pub max: u32,
}

impl DescriptorPoolChunk {
    pub fn new(device_info: &DeviceInfo) -> Self {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1000,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1000,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1000,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1000,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(1000)
            .pool_sizes(&pool_sizes);

        let pool = unsafe {
            device_info
                .logical_device
                .create_descriptor_pool(&pool_info, None)
                .expect("Failed to create descriptor pool")
        };

        Self {
            pool,
            used: 0,
            max: 1000,
        }
    }
}

pub struct DescriptorLayoutInfo {
    pub layout: vk::DescriptorSetLayout,
}

impl DescriptorLayoutInfo {
    pub fn new(device_info: &DeviceInfo, desc: DescriptorLayoutDesc) -> Self {
        let mut bindings = vec![];
        for binding in &desc.bindings {
            bindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(binding.binding)
                    .descriptor_type(map_descriptor_type(binding.descriptor_type))
                    .stage_flags(binding.stages.into())
                    .descriptor_count(binding.count),
            );
        }

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::empty())
            .bindings(&bindings);

        let layout = unsafe {
            device_info
                .logical_device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create layout")
        };

        Self { layout }
    }
}

fn map_descriptor_type(descriptor_type: DescriptorType) -> vk::DescriptorType {
    match descriptor_type {
        DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        DescriptorType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
        DescriptorType::Sampler => vk::DescriptorType::SAMPLER,
        DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
        DescriptorType::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
        DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
    }
}



pub struct AllocatedDescriptorSet {
    pub descriptor_set: vk::DescriptorSet,
    pub pool: vk::DescriptorPool,
}

impl AllocatedDescriptorSet {
    pub fn new(
        device_info: &DeviceInfo,
        layout: vk::DescriptorSetLayout,
        pool: &mut DescriptorPoolChunk,
    ) -> Self {
        let binding = [layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool.pool)
            .set_layouts(&binding);

        let descriptor_set = unsafe {
            device_info
                .logical_device
                .allocate_descriptor_sets(&allocate_info)
                .expect("Failed to allocate descriptor")[0]
        };

        pool.used += 1;

        Self {
            pool: pool.pool,
            descriptor_set,
        }
    }

    // pub fn update(&mut self, device_info: &DeviceInfo, write_descriptor_set: &[DescriptorWrite]) {
    //     let x = write_descriptor_set
    //         .iter()
    //         .map(|write_descriptor| match write_descriptor {
    //             DescriptorWrite::UniformBuffer(dst_binding, _) => {}
    //             DescriptorWrite::SampledImage(dst_binding, _, _) => {}
    //             DescriptorWrite::CombinedImageSampler(dst_binding, _, _) => {}
    //         });
    // 
    //     let buffer_info = vk::DescriptorBufferInfo::default()
    //         .buffer(camera_mvp_buffer.buffer)
    //         .offset(0)
    //         .range(mem::size_of::<CameraMvpUbo>() as u64);
    // 
    //     let dynamic_buffer_info = vk::DescriptorBufferInfo::default()
    //         .buffer(dynamic_model_buffer.buffer)
    //         .offset(0)
    //         .range(dynamic_alignment);
    // 
    //     let image_info = [vk::DescriptorImageInfo::default()
    //         .image_view(*texture_image_view)
    //         .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
    //         .sampler(*texture_sampler)];
    // 
    //     let mut write_descriptor_sets = vec![];
    // 
    //     write_descriptor_sets.push(
    //         vk::WriteDescriptorSet::default()
    //             .dst_set(descriptor_set)
    //             .dst_binding(0)
    //             .dst_array_element(0)
    //             .descriptor_type(DescriptorType::UNIFORM_BUFFER)
    //             .descriptor_count(1)
    //             .buffer_info(slice::from_ref(&buffer_info)),
    //     );
    // 
    //     write_descriptor_sets.push(
    //         vk::WriteDescriptorSet::default()
    //             .dst_set(descriptor_set)
    //             .dst_binding(1)
    //             .dst_array_element(0)
    //             .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
    //             .descriptor_count(1)
    //             .buffer_info(slice::from_ref(&dynamic_buffer_info)),
    //     );
    // 
    //     write_descriptor_sets.push(
    //         vk::WriteDescriptorSet::default()
    //             .dst_set(descriptor_set)
    //             .dst_binding(2)
    //             .dst_array_element(0)
    //             .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
    //             .descriptor_count(1)
    //             .image_info(image_info.as_slice()),
    //     );
    // 
    //     println!("Write descriptor sets");
    //     unsafe {
    //         device_info
    //             .logical_device
    //             .update_descriptor_sets(&write_descriptor_sets, &[]);
    //     }
    // }
}

// /// Uniform buffer count: 1 for camera, 1 for lighting
// const GLOBAL_UNIFORM_BUFFER_COUNT: usize = 20;
//
// /// Dynamic buffer count: 1 for model matrix
// const GLOBAL_DYNAMIC_UNIFORM_BUFFER_COUNT: usize = 20;
//
// /// Global image sampler count: 4 for albedo, normal, depth, shadow-map
// const GLOBAL_IMAGE_SAMPLER_COUNT: usize = 20;
//
// #[derive(Debug)]
// pub struct DescriptorInfo {
//     pub descriptor_pool: DescriptorPool,
//     pub descriptor_set_layout: DescriptorSetLayout,
//     pub descriptor_set: DescriptorSet,
// }
//
// impl DescriptorInfo {
//     pub(crate) fn new(
//         layout_description: DescriptorLayoutDescription,
//         device: &DeviceInfo,
//         sampler_count: u32,
//         buffer_count: u32,
//         dynamic_count: u32,
//     ) -> DescriptorInfo {
//         let descriptor_set_layout = layout_description.to_vk(device);
//         let descriptor_pool = Self::create_pool(device, sampler_count, buffer_count, dynamic_count);
//         let descriptor_set =
//             Self::create_descriptor_set(device, descriptor_set_layout, descriptor_pool);
//
//         DescriptorInfo {
//             descriptor_set,
//             descriptor_pool,
//             descriptor_set_layout,
//         }
//     }
//
//     fn create_pool(
//         device: &DeviceInfo,
//         sampler_count: u32,
//         buffer_count: u32,
//         dynamic_count: u32,
//     ) -> DescriptorPool {
//         let pool_sizes = [
//             DescriptorPoolSize::default()
//                 .descriptor_count(buffer_count)
//                 .ty(vk::DescriptorType::UNIFORM_BUFFER),
//             DescriptorPoolSize::default()
//                 .descriptor_count(dynamic_count)
//                 .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC),
//             DescriptorPoolSize::default()
//                 .descriptor_count(sampler_count)
//                 .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER),
//         ];
//
//         let create_info = DescriptorPoolCreateInfo::default()
//             .pool_sizes(&pool_sizes)
//             .max_sets(sampler_count + buffer_count + dynamic_count);
//
//         unsafe {
//             device
//                 .logical_device
//                 .create_descriptor_pool(&create_info, None)
//                 .expect("Failed to create global descriptor pool")
//         }
//     }
//
//     fn create_descriptor_set(
//         device: &DeviceInfo,
//         descriptor_layout: DescriptorSetLayout,
//         descriptor_pool: DescriptorPool,
//     ) -> DescriptorSet {
//         let binding = &[descriptor_layout];
//         let allocate_info = DescriptorSetAllocateInfo::default()
//             .descriptor_pool(descriptor_pool)
//             .set_layouts(binding);
//
//         unsafe {
//             device
//                 .logical_device
//                 .allocate_descriptor_sets(&allocate_info)
//                 .expect("failed to create descriptor set layout")[0]
//         }
//     }
// }
//
// #[derive(Clone, Debug, Copy)]
// pub struct DescriptorBindingInfo {
//     pub binding: u32,
//     pub descriptor_type: DescriptorType,
//     pub descriptor_count: u32,
//     pub stage_flags: vk::ShaderStageFlags,
// }
//
// #[derive(Debug)]
// pub struct DescriptorLayoutDescription {
//     pub bindings: Vec<DescriptorBindingInfo>,
// }
//
// impl DescriptorLayoutDescription {
//     pub(crate) fn to_vk(&self, device: &DeviceInfo) -> DescriptorSetLayout {
//         let bindings = self
//             .bindings
//             .iter()
//             .map(|binding| DescriptorSetLayoutBinding {
//                 binding: binding.clone().binding,
//                 descriptor_type: binding.clone().descriptor_type,
//                 descriptor_count: binding.clone().descriptor_count,
//                 stage_flags: binding.clone().stage_flags,
//                 ..Default::default()
//             })
//             .collect::<Vec<_>>();
//
//         let create_info = DescriptorSetLayoutCreateInfo::default()
//             .bindings(&*bindings)
//             .flags(DescriptorSetLayoutCreateFlags::empty());
//
//         unsafe {
//             device
//                 .logical_device
//                 .create_descriptor_set_layout(&create_info, None)
//                 .unwrap()
//         }
//     }
// }
//
// pub struct DescriptorManager {
//     pub global_pool: DescriptorPool,
//     pub global_gbuffer_layout: DescriptorSetLayout,
//     pub global_lighting_layout: DescriptorSetLayout,
//     pub global_shadow_map_layout: DescriptorSetLayout,
// }
//
// impl DescriptorManager {
//     pub fn new(device: &Device, max_frames: usize) -> Self {
//         let global_pool = Self::create_global_pool(device, max_frames);
//         let global_gbuffer_layout = Self::create_global_gbuffer_layout(device);
//         let global_lighting_layout = Self::create_global_lighting_layout(device);
//         let global_shadow_map_layout = Self::create_global_shadow_map_layout(device);
//
//         Self {
//             global_pool,
//             global_gbuffer_layout,
//             global_lighting_layout,
//             global_shadow_map_layout,
//         }
//     }
//
//     pub fn create_gbuffer_descriptor_set(&self, device: &Device) -> DescriptorSet {
//         let binding = [self.global_gbuffer_layout];
//         let allocate_info = DescriptorSetAllocateInfo::default()
//             .descriptor_pool(self.global_pool)
//             .set_layouts(&binding);
//
//         unsafe {
//             device
//                 .allocate_descriptor_sets(&allocate_info)
//                 .expect("failed to create descriptor set layout")[0]
//         }
//     }
//
//     pub fn create_lighting_descriptor_set(&self, device: &Device) -> DescriptorSet {
//         let binding = [self.global_lighting_layout];
//         let allocate_info = DescriptorSetAllocateInfo::default()
//             .descriptor_pool(self.global_pool)
//             .set_layouts(&binding);
//
//         unsafe {
//             device
//                 .allocate_descriptor_sets(&allocate_info)
//                 .expect("failed to create descriptor set layout")[0]
//         }
//     }
//
//     pub fn update_gbuffer_descriptor_set(
//         &self,
//         device_info: &DeviceInfo,
//         camera_mvp_buffer: &AllocatedBuffer,
//         dynamic_model_buffer: &AllocatedBuffer,
//         dynamic_alignment: u64,
//         texture_image_view: &ImageView,
//         texture_sampler: &vk::Sampler,
//         descriptor_set: DescriptorSet,
//     ) {
//         let buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(camera_mvp_buffer.buffer)
//             .offset(0)
//             .range(mem::size_of::<CameraMvpUbo>() as u64);
//
//         let dynamic_buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(dynamic_model_buffer.buffer)
//             .offset(0)
//             .range(dynamic_alignment);
//
//         let image_info = [vk::DescriptorImageInfo::default()
//             .image_view(*texture_image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(*texture_sampler)];
//
//         let mut write_descriptor_sets = vec![];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(0)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&buffer_info)),
//         );
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(1)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&dynamic_buffer_info)),
//         );
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(2)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(image_info.as_slice()),
//         );
//
//         println!("Write descriptor sets");
//         unsafe {
//             device_info
//                 .logical_device
//                 .update_descriptor_sets(&write_descriptor_sets, &[]);
//         }
//     }
//
//     pub fn update_dynamic_buffer_descriptor_sets(
//         &self,
//         device_info: &DeviceInfo,
//         dynamic_model_buffer: &AllocatedBuffer,
//         descriptor_set: DescriptorSet,
//         alignment: u64,
//     ) {
//         let dynamic_buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(dynamic_model_buffer.buffer)
//             .offset(0)
//             .range(alignment);
//
//         let mut write_descriptor_sets = vec![];
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(1)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&dynamic_buffer_info)),
//         );
//
//         unsafe {
//             device_info
//                 .logical_device
//                 .update_descriptor_sets(&write_descriptor_sets, &[]);
//         }
//     }
//
//     pub fn create_shadow_map_descriptor_set(&self, device: &Device) -> DescriptorSet {
//         let binding = [self.global_shadow_map_layout];
//         let allocate_info = DescriptorSetAllocateInfo::default()
//             .descriptor_pool(self.global_pool)
//             .set_layouts(&binding);
//
//         unsafe {
//             device
//                 .allocate_descriptor_sets(&allocate_info)
//                 .expect("failed to create descriptor set layout")[0]
//         }
//     }
//
//     pub fn update_shadow_map_descriptor_set(
//         &self,
//         device_info: &DeviceInfo,
//         shadow_map_buffer: &AllocatedBuffer,
//         descriptor_set: DescriptorSet,
//     ) {
//         let buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(shadow_map_buffer.buffer)
//             .offset(0)
//             .range(mem::size_of::<CascadeShadowUbo>() as u64 * 3);
//
//         let mut write_descriptor_sets = vec![];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(0)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&buffer_info)),
//         );
//
//         unsafe {
//             device_info
//                 .logical_device
//                 .update_descriptor_sets(&write_descriptor_sets, &[]);
//         }
//     }
//
//     pub fn update_lighting_descriptor_set(
//         &self,
//         device_info: &DeviceInfo,
//         lighting_buffer: &AllocatedBuffer,
//         albedo_image_view: &ImageView,
//         albedo_sampler: &vk::Sampler,
//         normal_image_view: &ImageView,
//         normal_sampler: &vk::Sampler,
//         depth_image_view: &ImageView,
//         depth_sampler: &vk::Sampler,
//         descriptor_set: DescriptorSet,
//         cascade: &Vec<ShadowCascade>,
//         shadow_map_buffer: &AllocatedBuffer,
//         camera_mvp_buffer: &AllocatedBuffer,
//         pos_image_view: &ImageView,
//         pos_sampler: &vk::Sampler,
//     ) {
//         let buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(lighting_buffer.buffer)
//             .offset(0)
//             .range(mem::size_of::<LightingUbo>() as u64);
//
//         let albedo_info = [vk::DescriptorImageInfo::default()
//             .image_view(*albedo_image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(*albedo_sampler)];
//
//         let normal_info = [vk::DescriptorImageInfo::default()
//             .image_view(*normal_image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(*normal_sampler)];
//
//         let depth_info = [vk::DescriptorImageInfo::default()
//             .image_view(*depth_image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(*depth_sampler)];
//
//         let mut write_descriptor_sets = vec![];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(0)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&buffer_info)),
//         );
//
//         let buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(shadow_map_buffer.buffer)
//             .offset(0)
//             .range(mem::size_of::<CascadeShadowUbo>() as u64 * (cascade.len()) as u64);
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(7)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&buffer_info)),
//         );
//
//         let buffer_info = vk::DescriptorBufferInfo::default()
//             .buffer(camera_mvp_buffer.buffer)
//             .offset(0)
//             .range(mem::size_of::<CameraMvpUbo>() as u64);
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(8)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .buffer_info(slice::from_ref(&buffer_info)),
//         );
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(1)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(albedo_info.as_slice()),
//         );
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(2)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(normal_info.as_slice()),
//         );
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(3)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(depth_info.as_slice()),
//         );
//
//         let depth_info = [vk::DescriptorImageInfo::default()
//             .image_view(cascade[0].shadow_depth_image.image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(cascade[0].cascade_sampler)];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(4)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(depth_info.as_slice()),
//         );
//
//         let depth_info = [vk::DescriptorImageInfo::default()
//             .image_view(cascade[1].shadow_depth_image.image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(cascade[1].cascade_sampler)];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(5)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(depth_info.as_slice()),
//         );
//
//         let depth_info = [vk::DescriptorImageInfo::default()
//             .image_view(cascade[2].shadow_depth_image.image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(cascade[2].cascade_sampler)];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(6)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(depth_info.as_slice()),
//         );
//
//         let pos_info = [vk::DescriptorImageInfo::default()
//             .image_view(*pos_image_view)
//             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//             .sampler(*pos_sampler)];
//
//         write_descriptor_sets.push(
//             vk::WriteDescriptorSet::default()
//                 .dst_set(descriptor_set)
//                 .dst_binding(9)
//                 .dst_array_element(0)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .image_info(pos_info.as_slice()),
//         );
//
//         unsafe {
//             device_info
//                 .logical_device
//                 .update_descriptor_sets(&write_descriptor_sets, &[]);
//         }
//     }
//
//     fn create_global_pool(device: &Device, max_frames: usize) -> DescriptorPool {
//         let pool_sizes = [
//             DescriptorPoolSize::default()
//                 .descriptor_count((max_frames * GLOBAL_UNIFORM_BUFFER_COUNT) as u32)
//                 .ty(DescriptorType::UNIFORM_BUFFER),
//             DescriptorPoolSize::default()
//                 .descriptor_count((max_frames * GLOBAL_DYNAMIC_UNIFORM_BUFFER_COUNT) as u32)
//                 .ty(DescriptorType::UNIFORM_BUFFER_DYNAMIC),
//             DescriptorPoolSize::default()
//                 .descriptor_count((max_frames * GLOBAL_IMAGE_SAMPLER_COUNT) as u32)
//                 .ty(DescriptorType::COMBINED_IMAGE_SAMPLER),
//         ];
//
//         let create_info = DescriptorPoolCreateInfo::default()
//             .pool_sizes(&pool_sizes)
//             .max_sets(max_frames as u32 * 4);
//
//         unsafe {
//             device
//                 .create_descriptor_pool(&create_info, None)
//                 .expect("Failed to create global descriptor pool")
//         }
//     }
//
//     fn create_global_lighting_layout(device: &Device) -> DescriptorSetLayout {
//         let bindings = [
//             // Lighting Data
//             DescriptorSetLayoutBinding::default()
//                 .binding(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             // Albedo Texture
//             DescriptorSetLayoutBinding::default()
//                 .binding(1)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             // Normal Texture
//             DescriptorSetLayoutBinding::default()
//                 .binding(2)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             // Depth Texture
//             DescriptorSetLayoutBinding::default()
//                 .binding(3)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             // Shadow map
//             DescriptorSetLayoutBinding::default()
//                 .binding(4)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             DescriptorSetLayoutBinding::default()
//                 .binding(5)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             DescriptorSetLayoutBinding::default()
//                 .binding(6)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             DescriptorSetLayoutBinding::default()
//                 .binding(7)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             DescriptorSetLayoutBinding::default()
//                 .binding(8)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//             DescriptorSetLayoutBinding::default()
//                 .binding(9)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//         ];
//
//         let create_info = DescriptorSetLayoutCreateInfo::default()
//             .bindings(&bindings)
//             .flags(DescriptorSetLayoutCreateFlags::empty());
//
//         unsafe {
//             device
//                 .create_descriptor_set_layout(&create_info, None)
//                 .expect("Failed to create global lighting descriptor set")
//         }
//     }
//
//     fn create_global_gbuffer_layout(device: &Device) -> DescriptorSetLayout {
//         let bindings = [
//             // Camera Data
//             DescriptorSetLayoutBinding::default()
//                 .binding(0)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::VERTEX),
//             // Model offset Texture
//             DescriptorSetLayoutBinding::default()
//                 .binding(1)
//                 .descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::VERTEX),
//             // TODO: Create Material system
//             // Texture
//             DescriptorSetLayoutBinding::default()
//                 .binding(2)
//                 .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::FRAGMENT),
//         ];
//
//         let create_info = DescriptorSetLayoutCreateInfo::default()
//             .bindings(&bindings)
//             .flags(DescriptorSetLayoutCreateFlags::empty());
//
//         unsafe {
//             device
//                 .create_descriptor_set_layout(&create_info, None)
//                 .expect("Failed to create global gBuffer descriptor set layout")
//         }
//     }
//
//     fn create_global_shadow_map_layout(device: &Device) -> DescriptorSetLayout {
//         let bindings = [DescriptorSetLayoutBinding::default()
//             .binding(0)
//             .descriptor_type(DescriptorType::UNIFORM_BUFFER)
//             .descriptor_count(1)
//             .stage_flags(vk::ShaderStageFlags::VERTEX)];
//
//         let create_info = DescriptorSetLayoutCreateInfo::default()
//             .bindings(&bindings)
//             .flags(DescriptorSetLayoutCreateFlags::empty());
//
//         unsafe {
//             device
//                 .create_descriptor_set_layout(&create_info, None)
//                 .expect("Failed to create global shadow map descriptor set layout")
//         }
//     }
// }
