use crate::frame_data::FrameData;
use crate::render_data::CameraRenderData;
use crate::render_scene::RenderScene;
use nalgebra_glm::{self as glm, Mat4, Vec3, Vec4};
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::descriptor::{
    DescriptorValue, DescriptorWriteDesc, SampledImageInfo, ShaderStage,
};
use rendering_backend::pipeline::{
    CompareOp, CullMode, DepthStencilDesc, FrontFace, PipelineDesc, PipelineHandle, PolygonMode,
    PushConstantDesc, RasterizationStateDesc, VertexInputDesc,
};

const CASCADE_COUNT: usize = 4;
const CASCADE_RESOLUTIONS: [u32; CASCADE_COUNT] = [2048, 2048, 1024, 1024];
const SHADOW_DISTANCE: f32 = 100.0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightingUbo {
    pub light_direction: Vec4,
    pub light_color: Vec4,
    pub ambient_light: Vec4,
    pub cascade_depths: Vec4,
}

#[derive(Clone, Copy)]
pub struct Cascade {
    pub view_proj: Mat4,
    pub depth: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ShadowPushConstants {
    object_index: u32,
    cascade_index: u32,
}

pub struct LightingRenderer {
    shadow_pipeline: PipelineHandle,
    lighting_pipeline: PipelineHandle,
}

impl LightingRenderer {
    pub fn new(vulkan_backend: &mut VulkanBackend, frame_data: &FrameData) -> Self {
        let shadow_pipeline_desc = PipelineDesc {
            vertex_shader: String::from("shadow"),
            fragment_shader: None,
            push_constant_ranges: vec![PushConstantDesc {
                stages: ShaderStage::VERTEX,
                offset: 0,
                size: size_of::<ShadowPushConstants>(),
            }],
            layout: vec![frame_data.shadow_descriptor_layout],
            color_attachments: vec![],
            depth_attachment: Some(
                *frame_data
                    .frame_images
                    .shadow_cascades
                    .first()
                    .expect("No shadow cascade image present!"),
            ),
            blend: None,
            depth_stencil: DepthStencilDesc {
                depth_test_enable: true,
                depth_write_enable: true,
                depth_compare_op: CompareOp::Less,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            rasterization: RasterizationStateDesc {
                cull_mode: CullMode::Front,
                depth_bias_enable: false,
                depth_clamp_enable: true,
                discard_enable: false,
                front_face: FrontFace::CounterClockwise,
                polygon_mode: PolygonMode::Fill,
            },
            vertex_input: VertexInputDesc {
                bindings: vec![],
                attributes: vec![],
            },
        };

        let shadow_pipeline = vulkan_backend.create_graphics_pipeline(shadow_pipeline_desc);

        let lighting_pipeline_desc = PipelineDesc {
            vertex_shader: String::from("quad"),
            fragment_shader: Some(String::from("lighting")),
            push_constant_ranges: vec![],
            layout: vec![frame_data.lighting_descriptor_layout],
            color_attachments: vec![frame_data.frame_images.draw_image],
            depth_attachment: None,
            blend: None,
            depth_stencil: DepthStencilDesc {
                depth_test_enable: false,
                depth_write_enable: false,
                depth_compare_op: CompareOp::Always,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            rasterization: RasterizationStateDesc {
                cull_mode: CullMode::None,
                depth_bias_enable: false,
                depth_clamp_enable: false,
                discard_enable: false,
                front_face: FrontFace::CounterClockwise,
                polygon_mode: PolygonMode::Fill,
            },
            vertex_input: VertexInputDesc {
                bindings: vec![],
                attributes: vec![],
            },
        };

        let lighting_pipeline = vulkan_backend.create_graphics_pipeline(lighting_pipeline_desc);

        Self {
            shadow_pipeline,
            lighting_pipeline,
        }
    }

    pub fn draw_frame(
        &self,
        vulkan_backend: &mut VulkanBackend,
        render_scene: &RenderScene,
        frame_data: &FrameData,
    ) {
        let light = match &render_scene.directional_light {
            Some(l) => l,
            None => return,
        };
        let camera = match &render_scene.camera_data {
            Some(c) => c,
            None => return,
        };
        
        let cascades = self.compute_cascades(camera, &light.direction);

        let cascade_matrices: Vec<Mat4> = cascades.iter().map(|c| c.view_proj).collect();
        vulkan_backend.update_buffer(frame_data.cascade_buffer, cascade_matrices.as_slice());

        let lighting_ubo = LightingUbo {
            light_direction: Vec4::new(
                light.direction.x,
                light.direction.y,
                light.direction.z,
                0.0,
            ),
            light_color: Vec4::new(light.color.x, light.color.y, light.color.z, light.intensity),
            ambient_light: Vec4::new(
                light.ambient_color.x,
                light.ambient_color.y,
                light.ambient_color.z,
                light.ambient_intensity,
            ),
            cascade_depths: Vec4::new(
                cascades.get(0).map_or(0.0, |c| c.depth),
                cascades.get(1).map_or(0.0, |c| c.depth),
                cascades.get(2).map_or(0.0, |c| c.depth),
                cascades.get(3).map_or(0.0, |c| c.depth),
            ),
        };
        vulkan_backend.update_buffer(frame_data.lighting_buffer, &[lighting_ubo]);

        self.update_lighting_descriptors(vulkan_backend, frame_data);

        for cascade_idx in 0..CASCADE_COUNT {
            let shadow_image = &frame_data.frame_images.shadow_cascades[cascade_idx];
            let res = CASCADE_RESOLUTIONS[cascade_idx];
            vulkan_backend.begin_rendering_with_extent(
                &[],
                Some(shadow_image),
                res,
                res,
            );

            vulkan_backend.bind_pipeline(self.shadow_pipeline);
            vulkan_backend
                .bind_descriptor_sets(&[frame_data.shadow_descriptor_set], self.shadow_pipeline);

            for (object_index, mesh_data) in render_scene.meshes.iter().enumerate() {
                let push = ShadowPushConstants {
                    object_index: object_index as u32,
                    cascade_index: cascade_idx as u32,
                };
                vulkan_backend.update_push_constants(
                    self.shadow_pipeline,
                    ShaderStage::VERTEX,
                    &[push],
                );

                vulkan_backend.bind_vertex_buffer(mesh_data.mesh_data.vertex_buffer);
                vulkan_backend.bind_index_buffer(mesh_data.mesh_data.index_buffer);
                vulkan_backend.draw_indexed(mesh_data.mesh_data.index_count as u32, 0);
            }

            vulkan_backend.end_rendering();
        }

        for cascade_idx in 0..CASCADE_COUNT {
            vulkan_backend
                .transition_image(frame_data.frame_images.shadow_cascades[cascade_idx], true);
        }

        vulkan_backend.transition_image(frame_data.frame_images.gbuffer_albedo, false);
        vulkan_backend.transition_image(frame_data.frame_images.gbuffer_normal, false);
        vulkan_backend.transition_image(frame_data.frame_images.gbuffer_depth, true);

        vulkan_backend.begin_rendering(&[frame_data.frame_images.draw_image], None);

        vulkan_backend.bind_pipeline(self.lighting_pipeline);
        vulkan_backend.bind_descriptor_sets(
            &[frame_data.lighting_descriptor_set],
            self.lighting_pipeline,
        );
        vulkan_backend.draw(3);

        vulkan_backend.end_rendering();
    }

    fn update_lighting_descriptors(
        &self,
        vulkan_backend: &mut VulkanBackend,
        frame_data: &FrameData,
    ) {
        let writes = vec![
            DescriptorWriteDesc {
                binding: 0,
                value: DescriptorValue::UniformBuffer(frame_data.lighting_buffer),
            },
            DescriptorWriteDesc {
                binding: 1,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.gbuffer_albedo,
                    sampler: frame_data.basic_sampler,
                }),
            },
            DescriptorWriteDesc {
                binding: 2,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.gbuffer_normal,
                    sampler: frame_data.basic_sampler,
                }),
            },
            DescriptorWriteDesc {
                binding: 3,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.gbuffer_depth,
                    sampler: frame_data.basic_sampler,
                }),
            },
            DescriptorWriteDesc {
                binding: 4,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.shadow_cascades[0],
                    sampler: frame_data.shadow_sampler,
                }),
            },
            DescriptorWriteDesc {
                binding: 5,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.shadow_cascades[1],
                    sampler: frame_data.shadow_sampler,
                }),
            },
            DescriptorWriteDesc {
                binding: 6,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.shadow_cascades[2],
                    sampler: frame_data.shadow_sampler,
                }),
            },
            DescriptorWriteDesc {
                binding: 7,
                value: DescriptorValue::UniformBuffer(frame_data.cascade_buffer),
            },
            DescriptorWriteDesc {
                binding: 8,
                value: DescriptorValue::UniformBuffer(frame_data.camera_buffer),
            },
            DescriptorWriteDesc {
                binding: 9,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: frame_data.frame_images.shadow_cascades[3],
                    sampler: frame_data.shadow_sampler,
                }),
            },
        ];

        vulkan_backend.update_descriptor_set(frame_data.lighting_descriptor_set, &writes);
    }

    fn compute_cascades(&self, camera: &CameraRenderData, light_dir: &Vec3) -> Vec<Cascade> {
        let near = camera.near_clip;
        let far = camera.far_clip.min(SHADOW_DISTANCE);
        let lambda = 0.9f32;

        let mut splits = vec![0.0f32; CASCADE_COUNT + 1];
        splits[0] = near;
        splits[CASCADE_COUNT] = far;

        for i in 1..CASCADE_COUNT {
            let idm = i as f32 / CASCADE_COUNT as f32;
            let log = near * (far / near).powf(idm);
            let uniform = near + (far - near) * idm;
            splits[i] = log * lambda + uniform * (1.0 - lambda);
        }

        let mut cascades = Vec::with_capacity(CASCADE_COUNT);

        for i in 0..CASCADE_COUNT {
            let split_near = splits[i];
            let split_far = splits[i + 1];

            // Build sub-frustum projection
            let sub_proj = glm::perspective_rh_zo(
                camera.aspect_ratio,
                camera.fov.to_radians(),
                split_near,
                split_far,
            );

            let sub_inv_vp = glm::inverse(&(sub_proj * camera.view));

            // NDC corners of the frustum
            let ndc_corners = [
                Vec4::new(-1.0, -1.0, 0.0, 1.0),
                Vec4::new(1.0, -1.0, 0.0, 1.0),
                Vec4::new(1.0, 1.0, 0.0, 1.0),
                Vec4::new(-1.0, 1.0, 0.0, 1.0),
                Vec4::new(-1.0, -1.0, 1.0, 1.0),
                Vec4::new(1.0, -1.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(-1.0, 1.0, 1.0, 1.0),
            ];

            // Frustum to world space
            let mut world_corners = [Vec3::zeros(); 8];
            for (j, ndc) in ndc_corners.iter().enumerate() {
                let world = sub_inv_vp * ndc;
                world_corners[j] =
                    Vec3::new(world.x / world.w, world.y / world.w, world.z / world.w);
            }
            
            let mut center = Vec3::zeros();
            for corner in &world_corners {
                center += corner;
            }
            center /= 8.0;
            
            let mut radius = 0.0f32;
            for corner in &world_corners {
                let dist = glm::length(&(corner - center));
                radius = radius.max(dist);
            }
            // Round texel size
            radius = (radius * 16.0).ceil() / 16.0;

            let light_dir_norm = glm::normalize(light_dir);
            let light_view = glm::look_at(
                &(center + light_dir_norm * radius),
                &center,
                &Vec3::new(0.0, 1.0, 0.0),
            );
            let light_proj = glm::ortho_rh_zo(-radius, radius, -radius, radius, 0.0, 2.0 * radius);

            let mut shadow_vp = light_proj * light_view;

            // Texel snapping
            let snap_res = CASCADE_RESOLUTIONS[i] as f32;
            let shadow_origin = shadow_vp * Vec4::new(0.0, 0.0, 0.0, 1.0);
            let shadow_origin_snapped = Vec4::new(
                (shadow_origin.x * snap_res / 2.0).round() / (snap_res / 2.0),
                (shadow_origin.y * snap_res / 2.0).round() / (snap_res / 2.0),
                shadow_origin.z,
                shadow_origin.w,
            );
            let offset = shadow_origin_snapped - shadow_origin;

            shadow_vp[(0, 3)] += offset.x;
            shadow_vp[(1, 3)] += offset.y;

            cascades.push(Cascade {
                view_proj: shadow_vp,
                depth: split_far,
            });
        }

        cascades
    }
}
