use crate::frame_data::FrameData;
use nalgebra_glm::{Mat4, Vec2, Vec3};
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use rendering_backend::camera::CameraMvpUbo;
use rendering_backend::descriptor::{
    DescriptorBinding, DescriptorLayoutDesc, DescriptorLayoutHandle, DescriptorSetHandle,
    DescriptorType, DescriptorValue, DescriptorWriteDesc, ShaderStage,
};
use rendering_backend::memory::MemoryHint;
use rendering_backend::pipeline::{
    BlendAttachmentDesc, BlendFactor, BlendOp, BlendStateDesc, ColorWriteMask, CompareOp, CullMode,
    DepthStencilDesc, FrontFace, PipelineDesc, PipelineHandle, PolygonMode, PrimitiveTopology,
    RasterizationStateDesc, VertexInputDesc,
};
use rendering_backend::vertex::Vertex;

/// Axis-aligned bounding box passed to the debug renderer for wireframe drawing.
pub struct DebugBox {
    pub min: Vec3,
    pub max: Vec3,
}

/// Renders wireframe AABB overlays on the final draw image. Toggled at runtime
/// with `toggle()`. When disabled, `draw_frame` is a no-op.
///
/// The pipeline and descriptor set are created lazily on the first draw call.
/// `camera_buffer` and `model_buffer` are pre-allocated in `new()`. The model
/// matrix is always identity because AABBs are already in world space.
pub struct AabbDebugRenderer {
    pub enabled: bool,
    pipeline: Option<PipelineHandle>,
    vertex_buffer: Option<BufferHandle>,
    camera_buffer: BufferHandle,
    model_buffer: BufferHandle,
    descriptor_layout: Option<DescriptorLayoutHandle>,
    descriptor_set: Option<DescriptorSetHandle>,
}

impl AabbDebugRenderer {
    /// Creates the renderer and pre-allocates the camera and model uniform buffers.
    pub fn new(vulkan_backend: &mut VulkanBackend) -> Self {
        let camera_buffer = vulkan_backend.create_buffer::<CameraMvpUbo>(
            BufferDesc {
                size: std::mem::size_of::<CameraMvpUbo>(),
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            None,
        );

        let identity = Mat4::identity();
        let model_buffer = vulkan_backend.create_buffer::<Mat4>(
            BufferDesc {
                size: std::mem::size_of::<Mat4>(),
                usage: BufferUsageFlags::UNIFORM,
                memory_hint: MemoryHint::CPUWritable,
            },
            Some(&[identity]),
        );

        Self {
            enabled: false,
            pipeline: None,
            vertex_buffer: None,
            camera_buffer,
            model_buffer,
            descriptor_layout: None,
            descriptor_set: None,
        }
    }

    /// Toggles the wireframe overlay on or off.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Draws wireframe boxes for each entry in `aabbs` on top of the current
    /// draw image. Does nothing if the renderer is disabled or `aabbs` is empty.
    pub fn draw_frame(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        aabbs: &[DebugBox],
        frame_data: &FrameData,
        camera: CameraMvpUbo,
    ) {
        if !self.enabled || aabbs.is_empty() {
            return;
        }

        vulkan_backend.update_buffer(self.camera_buffer, &[camera]);

        let (pipeline, descriptor_set) = self.get_or_create_pipeline(vulkan_backend, frame_data);

        let vertices = aabb_to_line_vertices(aabbs);
        let vertex_count = vertices.len() as u32;

        let needed_size = std::mem::size_of::<Vertex>() * vertices.len();
        let needs_new_buffer = self.vertex_buffer.is_none()
            || vulkan_backend
                .buffer_size(self.vertex_buffer.unwrap())
                < needed_size;

        if needs_new_buffer {
            let vb = vulkan_backend.create_buffer::<Vertex>(
                BufferDesc {
                    size: needed_size,
                    usage: BufferUsageFlags::VERTEX_BUFFER,
                    memory_hint: MemoryHint::CPUWritable,
                },
                Some(&vertices),
            );
            self.vertex_buffer = Some(vb);
        } else {
            vulkan_backend.update_buffer(self.vertex_buffer.unwrap(), &vertices);
        }

        vulkan_backend.begin_rendering_load(&[frame_data.frame_images.draw_image]);
        vulkan_backend.bind_pipeline(pipeline);
        vulkan_backend.bind_descriptor_sets(&[descriptor_set], pipeline);
        vulkan_backend.bind_vertex_buffer(self.vertex_buffer.unwrap());
        vulkan_backend.draw(vertex_count);
        vulkan_backend.end_rendering();
    }

    fn get_or_create_pipeline(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        frame_data: &FrameData,
    ) -> (PipelineHandle, DescriptorSetHandle) {
        if let (Some(pipeline), Some(descriptor_set)) = (self.pipeline, self.descriptor_set) {
            return (pipeline, descriptor_set);
        }

        let layout = vulkan_backend.create_descriptor_layout(DescriptorLayoutDesc {
            bindings: vec![
                DescriptorBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UniformBuffer,
                    count: 1,
                    stages: ShaderStage::VERTEX,
                },
                DescriptorBinding {
                    binding: 1,
                    descriptor_type: DescriptorType::UniformBuffer,
                    count: 1,
                    stages: ShaderStage::VERTEX,
                },
            ],
        });

        let descriptor_set = vulkan_backend.allocate_descriptor_set(layout);
        vulkan_backend.update_descriptor_set(
            descriptor_set,
            &[
                DescriptorWriteDesc {
                    binding: 0,
                    value: DescriptorValue::UniformBuffer(self.camera_buffer),
                },
                DescriptorWriteDesc {
                    binding: 1,
                    value: DescriptorValue::UniformBuffer(self.model_buffer),
                },
            ],
        );

        let pipeline_desc = PipelineDesc {
            vertex_shader: String::from("line_debug_vert"),
            fragment_shader: Some(String::from("line_debug_frag")),
            topology: PrimitiveTopology::LineList,
            color_attachments: vec![frame_data.frame_images.draw_image],
            depth_attachment: None,
            depth_stencil: DepthStencilDesc {
                depth_test_enable: false,
                depth_write_enable: false,
                depth_compare_op: CompareOp::Always,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            rasterization: RasterizationStateDesc {
                polygon_mode: PolygonMode::Fill,
                cull_mode: CullMode::None,
                front_face: FrontFace::CounterClockwise,
                depth_clamp_enable: false,
                depth_bias_enable: false,
                discard_enable: false,
            },
            blend: Some(BlendStateDesc {
                logic_op_enable: false,
                attachments: vec![BlendAttachmentDesc {
                    blend_enable: false,
                    src_color_blend: BlendFactor::One,
                    dst_color_blend: BlendFactor::Zero,
                    color_blend_op: BlendOp::Add,
                    src_alpha_blend: BlendFactor::One,
                    dst_alpha_blend: BlendFactor::Zero,
                    alpha_blend_op: BlendOp::Add,
                    color_write_mask: ColorWriteMask::ALL,
                }],
            }),
            layout: vec![layout],
            push_constant_ranges: vec![],
            vertex_input: VertexInputDesc {
                bindings: vec![],
                attributes: vec![],
            },
        };

        let pipeline = vulkan_backend.create_graphics_pipeline(pipeline_desc);

        self.descriptor_layout = Some(layout);
        self.descriptor_set = Some(descriptor_set);
        self.pipeline = Some(pipeline);

        (pipeline, descriptor_set)
    }
}

fn aabb_to_line_vertices(aabbs: &[DebugBox]) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(aabbs.len() * 24);
    let green = Vec3::new(0.0f32, 1.0, 0.0);
    let tex = Vec2::new(0.0f32, 0.0);
    let normal = Vec3::new(0.0f32, 0.0, 0.0);

    for aabb in aabbs {
        let bounds_min = aabb.min;
        let bounds_max = aabb.max;

        // 8 corners
        let c = [
            Vec3::new(bounds_min.x, bounds_min.y, bounds_min.z), // 0 bottom-front-left
            Vec3::new(bounds_max.x, bounds_min.y, bounds_min.z), // 1 bottom-front-right
            Vec3::new(bounds_max.x, bounds_min.y, bounds_max.z), // 2 bottom-back-right
            Vec3::new(bounds_min.x, bounds_min.y, bounds_max.z), // 3 bottom-back-left
            Vec3::new(bounds_min.x, bounds_max.y, bounds_min.z), // 4 top-front-left
            Vec3::new(bounds_max.x, bounds_max.y, bounds_min.z), // 5 top-front-right
            Vec3::new(bounds_max.x, bounds_max.y, bounds_max.z), // 6 top-back-right
            Vec3::new(bounds_min.x, bounds_max.y, bounds_max.z), // 7 top-back-left
        ];

        // 12 edges: 4 bottom, 4 top, 4 pillars
        let edges: [(usize, usize); 12] = [
            (0, 1), (1, 2), (2, 3), (3, 0), // bottom face
            (4, 5), (5, 6), (6, 7), (7, 4), // top face
            (0, 4), (1, 5), (2, 6), (3, 7), // pillars
        ];

        for (a, b) in edges {
            vertices.push(Vertex {
                pos: c[a],
                color: green,
                tex_coord: tex,
                normal,
                texture_index: 0,
            });
            vertices.push(Vertex {
                pos: c[b],
                color: green,
                tex_coord: tex,
                normal,
                texture_index: 0,
            });
        }
    }

    vertices
}
