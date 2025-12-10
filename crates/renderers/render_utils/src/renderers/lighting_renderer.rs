// use crate::renderers::geometry_renderer::SceneData;
// use vulkan_backend::descriptor::{
//     DescriptorBindingInfo, DescriptorInfo, DescriptorLayoutDescription,
// };
// use vulkan_backend::graphics_pipeline::{PipelineInfo, PipelineLayoutDescription};
// use vulkan_backend::image_util::AllocatedImage;
// use vulkan_backend::vulkan_backend::VulkanBackend;
// use vulkan_backend::{DescriptorType, ShaderStageFlags};
// 
// pub struct LightingRenderer {
//     pipeline: PipelineInfo,
//     descriptor: DescriptorInfo,
// }
// 
// impl LightingRenderer {
//     pub fn new(vulkan_backend: &VulkanBackend) -> Self {
//         let descriptor = Self::create_descriptor(vulkan_backend);
//         let pipeline = Self::create_pipeline(vulkan_backend, &descriptor);
// 
//         Self {
//             descriptor,
//             pipeline,
//         }
//     }
// 
//     pub fn render(
//         &self,
//         vulkan_backend: &VulkanBackend,
//         scene_data: &Vec<SceneData>,
//         albedo_image: &AllocatedImage,
//         normal_image: &AllocatedImage,
//         depth_image: &AllocatedImage,
//     ) {
//         vulkan_backend.begin_rendering(&[albedo_image, normal_image], &[depth_image]);
// 
//         for scene_data in scene_data {
//             vulkan_backend.draw_indexed(
//                 &scene_data.mesh_data,
//                 &self.descriptor,
//                 &self.pipeline,
//                 scene_data.model,
//                 0,
//                 0,
//             )
//         }
//     }
// 
//     fn create_descriptor(vulkan_backend: &VulkanBackend) -> DescriptorInfo {
//         let desc = DescriptorLayoutDescription {
//             bindings: vec![
//                 // Lighting Data
//                 DescriptorBindingInfo {
//                     binding: 0,
//                     descriptor_type: DescriptorType::UNIFORM_BUFFER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Albedo Texture
//                 DescriptorBindingInfo {
//                     binding: 1,
//                     descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Normal Texture
//                 DescriptorBindingInfo {
//                     binding: 2,
//                     descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Depth Texture
//                 DescriptorBindingInfo {
//                     binding: 3,
//                     descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Shadow map
//                 DescriptorBindingInfo {
//                     binding: 4,
//                     descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Shadow map
//                 DescriptorBindingInfo {
//                     binding: 5,
//                     descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Shadow map
//                 DescriptorBindingInfo {
//                     binding: 6,
//                     descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Cascade Projections
//                 DescriptorBindingInfo {
//                     binding: 7,
//                     descriptor_type: DescriptorType::UNIFORM_BUFFER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//                 // Camera
//                 DescriptorBindingInfo {
//                     binding: 8,
//                     descriptor_type: DescriptorType::UNIFORM_BUFFER,
//                     descriptor_count: 1,
//                     stage_flags: ShaderStageFlags::FRAGMENT,
//                 },
//             ],
//         };
// 
//         vulkan_backend.create_descriptor_info(desc, 6, 3, 0)
//     }
// 
//     fn create_pipeline(
//         vulkan_backend: &VulkanBackend,
//         descriptor_info: &DescriptorInfo,
//     ) -> PipelineInfo {
//         let pipeline_layout_description = PipelineLayoutDescription {
//             fragment_shader_path: String::from("lighting.frag"),
//             vertex_shader_path: String::from("lighting.vert"),
//             descriptor_info,
//             push_constant_ranges: vec![],
//         };
// 
//         vulkan_backend.create_lighting_pipeline_info(pipeline_layout_description)
//     }
// }
