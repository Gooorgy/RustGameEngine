use config::config::WindowResolution;
use imgui::DrawVert;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::buffer::{BufferDesc, BufferHandle, BufferUsageFlags};
use rendering_backend::descriptor::{DescriptorBinding, DescriptorLayoutDesc, DescriptorSetHandle, DescriptorType, DescriptorValue, DescriptorWriteDesc, SampledImageInfo, ShaderStage};
use rendering_backend::image::{
    GpuImageHandle, ImageAspect, ImageDesc, ImageUsageFlags, TextureFormat,
};
use rendering_backend::memory::MemoryHint;
use rendering_backend::pipeline::FrontFace::CounterClockwise;
use rendering_backend::pipeline::{
    BlendAttachmentDesc, BlendFactor, BlendOp, BlendStateDesc,
    ColorWriteMask, CompareOp, CullMode, DepthStencilDesc, PipelineDesc, PipelineHandle,
    PolygonMode, PrimitiveTopology, PushConstantDesc, RasterizationStateDesc, VertexAttributeDesc,
    VertexBindingDesc, VertexFormat, VertexInputDesc, VertexInputRate,
};
use std::mem::offset_of;
use rendering_backend::sampler::SamplerHandle;

pub struct ImuiRenderer {
    pub ctx: imgui::Context,
    pub pipeline: PipelineHandle,
    pub font_descriptor: DescriptorSetHandle,
    pub font_image: GpuImageHandle,
    pub vertex_buffer: BufferHandle,
    pub index_buffer: BufferHandle,
}

impl ImuiRenderer {
    pub fn new(vulkan_backend: &mut VulkanBackend, draw_image: GpuImageHandle, image_sampler: SamplerHandle) -> Self {
        let mut ctx = imgui::Context::create();
        ctx.set_ini_filename(None);
        ctx.io_mut()
            .backend_flags
            .insert(imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET);

        let atlas = ctx.fonts().build_rgba32_texture();
        let font_image = vulkan_backend.create_image(ImageDesc {
            width: atlas.width,
            height: atlas.height,
            depth: 1,
            format: TextureFormat::R8g8b8a8Unorm,
            clear_value: None,
            array_layers: 1,
            mip_levels: 1,
            is_cubemap: false,
            aspect: ImageAspect::Color,
            usage: ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST,
        });

        vulkan_backend.update_image_data(font_image, atlas.data);

        let layout_desc = vulkan_backend.create_descriptor_layout(DescriptorLayoutDesc {
            bindings: vec![DescriptorBinding {
                binding: 0,
                descriptor_type: DescriptorType::CombinedImageSampler,
                count: 1,
                stages: ShaderStage::FRAGMENT,
            }],
        });

        let desc = vulkan_backend.allocate_descriptor_set(layout_desc);

        let write_desc = DescriptorWriteDesc {
            binding: 0,
            value: DescriptorValue::SampledImage(SampledImageInfo {
                image: font_image,
                sampler: image_sampler,
            }),
        };

        vulkan_backend.update_descriptor_set(desc, &[write_desc]);


        let pipeline = vulkan_backend.create_graphics_pipeline(PipelineDesc {
            vertex_shader: Vec::from(include_bytes!("./shaders/imgui.vert.spv")),
            fragment_shader: Some(Vec::from(include_bytes!("./shaders/imgui.frag.spv"))),
            color_attachments: vec![draw_image],
            depth_attachment: None,
            layout: vec![layout_desc],
            vertex_input: VertexInputDesc {
                bindings: vec![VertexBindingDesc {
                    binding: 0,
                    input_rate: VertexInputRate::Vertex,
                    stride: size_of::<DrawVert>() as u32,
                }],
                attributes: vec![
                    VertexAttributeDesc {
                        location: 0,
                        binding: 0,
                        format: VertexFormat::Float32x2,
                        offset: offset_of!(DrawVert, pos) as u32,
                    },
                    VertexAttributeDesc {
                        location: 1,
                        binding: 0,
                        format: VertexFormat::Float32x2,
                        offset: offset_of!(DrawVert, uv) as u32,
                    },
                    VertexAttributeDesc {
                        location: 2,
                        binding: 0,
                        format: VertexFormat::Unorm8x4,
                        offset: offset_of!(DrawVert, col) as u32,
                    },
                ],
            },
            blend: Some(BlendStateDesc {
                logic_op_enable: false,
                attachments: vec![BlendAttachmentDesc {
                    src_color_blend: BlendFactor::SrcAlpha,
                    dst_color_blend: BlendFactor::OneMinusSrcAlpha,
                    color_blend_op: BlendOp::Add,
                    src_alpha_blend: BlendFactor::One,
                    dst_alpha_blend: BlendFactor::OneMinusSrcAlpha,
                    alpha_blend_op: BlendOp::Add,
                    blend_enable: true,
                    color_write_mask: ColorWriteMask::ALL,
                }],
            }),
            rasterization: RasterizationStateDesc {
                polygon_mode: PolygonMode::Fill,
                cull_mode: CullMode::None,
                front_face: CounterClockwise,
                discard_enable: false,
                depth_clamp_enable: false,
                depth_bias_enable: false,
            },
            depth_stencil: DepthStencilDesc {
                depth_test_enable: true,
                depth_write_enable: true,
                depth_compare_op: CompareOp::Less,
                depth_bounds_test_enable: false,
                stencil_test_enable: false,
            },
            topology: PrimitiveTopology::TriangleList,
            push_constant_ranges: vec![PushConstantDesc {
                stages: ShaderStage::VERTEX,
                offset: 0,
                size: 16,
            }],
        });

        const MAX_VERTS: usize = 200_000;
        const MAX_INDICES: usize = 600_000;

        let vertex_buffer = vulkan_backend.create_buffer::<[f32; 3]>(
            BufferDesc {
                usage: BufferUsageFlags::INDEX_BUFFER,
                memory_hint: MemoryHint::CPUWritable,
                size: MAX_VERTS,
            },
            None,
        );

        let index_buffer = vulkan_backend.create_buffer::<u16>(
            BufferDesc {
                usage: BufferUsageFlags::INDEX_BUFFER,
                memory_hint: MemoryHint::CPUWritable,
                size: MAX_INDICES,
            },
            None,
        );

        Self {
            ctx,
            font_image,
            font_descriptor: desc,
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn draw_frame(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        extent: [f32; 2],
        delta_time: f32,
        draw_image: GpuImageHandle,
    ) {
        let io = self.ctx.io_mut();
        io.display_size = extent;
        io.display_framebuffer_scale = [1.0, 1.0];
        io.delta_time = delta_time;

        let ui = self.ctx.new_frame();
        ui.show_demo_window(&mut true);
        let draw_data = self.ctx.render();

        let mut vtx = vec![];
        let mut idx = vec![];
        for list in draw_data.draw_lists() {
            vtx.extend_from_slice(list.vtx_buffer());
            idx.extend_from_slice(list.idx_buffer());
        }

        vulkan_backend.update_buffer(self.vertex_buffer, vtx.as_slice());
        vulkan_backend.update_buffer(self.index_buffer, idx.as_slice());

        vulkan_backend.begin_rendering_load(&[draw_image]);
        vulkan_backend.bind_pipeline(self.pipeline);
        vulkan_backend.bind_descriptor_sets(&[self.font_descriptor], self.pipeline);

        let scale = [
            2.0 / draw_data.display_size[0],
            2.0 / draw_data.display_size[1],
        ];
        let translate = [
            -1.0 - draw_data.display_pos[0] * scale[0],
            -1.0 - draw_data.display_pos[1] * scale[1],
        ];
        vulkan_backend.update_push_constants(
            self.pipeline,
            ShaderStage::VERTEX,
            &[scale, translate],
        );
        vulkan_backend.bind_vertex_buffer(self.vertex_buffer);
        vulkan_backend.bind_index_buffer_u16(self.index_buffer);
        let (w, h) = (extent[0] as f32, extent[1] as f32);
        let (mut gv, mut gi) = (0u32, 0u32);
        for list in draw_data.draw_lists() {
            for command in list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = command {
                    let r = cmd_params.clip_rect;
                    let (x0, y0) = (r[0].max(0.0), r[1].max(0.0));
                    let (x1, y1) = (r[2].min(w), r[3].min(h));
                    if x1 <= x0 || y1 <= y0 {
                        continue;
                    }

                    vulkan_backend.draw_indexed(
                        count as u32,
                        gi + cmd_params.idx_offset as u32,
                        gv as i32 + cmd_params.vtx_offset as i32,
                    )
                }
            }
            gv += list.vtx_buffer().len() as u32;
            gi += list.idx_buffer().len() as u32;
        }

        vulkan_backend.end_rendering();
    }
}

pub fn init_imgui(res: WindowResolution) {
    let mut ctx = imgui::Context::create();
    ctx.set_ini_filename(None);
    ctx.io_mut()
        .backend_flags
        .insert(imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET);

    let ui = ctx.new_frame();
    ui.show_demo_window(&mut true);
    let draw_data = ctx.render();
}
