use crate::descriptor::{DescriptorLayoutHandle, ShaderStage};
use crate::image::GpuImageHandle;

#[derive(Copy, Clone, Debug)]
pub struct PipelineHandle(pub usize);

pub struct PipelineDesc {
    pub vertex_shader: String,
    pub fragment_shader: Option<String>,
    pub layout: Vec<DescriptorLayoutHandle>,
    pub vertex_input: VertexInputDesc,
    pub rasterization: RasterizationStateDesc,
    pub blend: BlendStateDesc,
    pub depth_stencil: DepthStencilDesc,
    pub color_attachments: Vec<GpuImageHandle>,
    pub depth_attachment: Option<GpuImageHandle>,
    pub push_constant_ranges: Vec<PushConstantDesc>,
}

pub struct PushConstantDesc {
    pub stages: ShaderStage,
    pub offset: u32,
    pub size: usize,
}

#[derive(Clone, Debug)]
pub struct VertexInputDesc {
    pub bindings: Vec<VertexBindingDesc>,
    pub attributes: Vec<VertexAttributeDesc>,
}

#[derive(Clone, Debug)]
pub struct VertexBindingDesc {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

#[derive(Clone, Debug)]
pub enum VertexInputRate {
    Vertex,
    Instance,
}

#[derive(Clone, Debug)]
pub struct VertexAttributeDesc {
    pub location: u32,
    pub binding: u32,
    pub format: VertexFormat,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum VertexFormat {
    Float32x1,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32x1,
    Uint32x2,
    Uint32x3,
    Uint32x4,
}

#[derive(Clone, Debug)]
pub struct RasterizationStateDesc {
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub depth_clamp_enable: bool,
    pub depth_bias_enable: bool,
    pub discard_enable: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

#[derive(Clone, Copy, Debug)]
pub enum CullMode {
    None,
    Front,
    Back,
    FrontAndBack,
}

#[derive(Clone, Copy, Debug)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Debug)]
pub struct BlendStateDesc {
    pub logic_op_enable: bool,
    pub attachments: Vec<BlendAttachmentDesc>,
}

#[derive(Clone, Debug)]
pub struct BlendAttachmentDesc {
    pub blend_enable: bool,
    pub src_color_blend: BlendFactor,
    pub dst_color_blend: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend: BlendFactor,
    pub dst_alpha_blend: BlendFactor,
    pub alpha_blend_op: BlendOp,
    pub color_write_mask: ColorWriteMask,
}

#[derive(Clone, Copy, Debug)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}

#[derive(Clone, Copy, Debug)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct ColorWriteMask: u32 {
        const R = 0x1;
        const G = 0x2;
        const B = 0x4;
        const A = 0x8;
        const ALL = Self::R.bits() | Self::G.bits() | Self::B.bits() | Self::A.bits();
    }
}

#[derive(Clone, Debug)]
pub struct DepthStencilDesc {
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: CompareOp,
    pub depth_bounds_test_enable: bool,
    pub stencil_test_enable: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}
