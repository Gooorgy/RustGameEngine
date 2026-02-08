use crate::descriptor::ShaderStage;
use crate::pipeline::{
    BlendFactor, BlendOp, ColorWriteMask, CompareOp, CullMode, FrontFace, PolygonMode, VertexFormat,
};
use crate::sampler::{Filter, SamplerAddressMode};
use ash::vk;

impl From<VertexFormat> for vk::Format {
    fn from(fmt: VertexFormat) -> Self {
        match fmt {
            VertexFormat::Float32x1 => vk::Format::R32_SFLOAT,
            VertexFormat::Float32x2 => vk::Format::R32G32_SFLOAT,
            VertexFormat::Float32x3 => vk::Format::R32G32B32_SFLOAT,
            VertexFormat::Float32x4 => vk::Format::R32G32B32A32_SFLOAT,

            VertexFormat::Uint32x1 => vk::Format::R32_UINT,
            VertexFormat::Uint32x2 => vk::Format::R32G32_UINT,
            VertexFormat::Uint32x3 => vk::Format::R32G32B32_UINT,
            VertexFormat::Uint32x4 => vk::Format::R32G32B32A32_UINT,
        }
    }
}

impl From<PolygonMode> for vk::PolygonMode {
    fn from(mode: PolygonMode) -> Self {
        match mode {
            PolygonMode::Fill => vk::PolygonMode::FILL,
            PolygonMode::Line => vk::PolygonMode::LINE,
            PolygonMode::Point => vk::PolygonMode::POINT,
        }
    }
}

impl From<CullMode> for vk::CullModeFlags {
    fn from(mode: CullMode) -> Self {
        match mode {
            CullMode::None => vk::CullModeFlags::NONE,
            CullMode::Front => vk::CullModeFlags::FRONT,
            CullMode::Back => vk::CullModeFlags::BACK,
            CullMode::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
        }
    }
}

impl From<FrontFace> for vk::FrontFace {
    fn from(face: FrontFace) -> Self {
        match face {
            FrontFace::Clockwise => vk::FrontFace::CLOCKWISE,
            FrontFace::CounterClockwise => vk::FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

impl From<BlendFactor> for vk::BlendFactor {
    fn from(f: BlendFactor) -> Self {
        match f {
            BlendFactor::Zero => vk::BlendFactor::ZERO,
            BlendFactor::One => vk::BlendFactor::ONE,

            BlendFactor::SrcColor => vk::BlendFactor::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => vk::BlendFactor::DST_COLOR,
            BlendFactor::OneMinusDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,

            BlendFactor::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => vk::BlendFactor::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
        }
    }
}

impl From<BlendOp> for vk::BlendOp {
    fn from(op: BlendOp) -> Self {
        match op {
            BlendOp::Add => vk::BlendOp::ADD,
            BlendOp::Subtract => vk::BlendOp::SUBTRACT,
            BlendOp::ReverseSubtract => vk::BlendOp::REVERSE_SUBTRACT,
            BlendOp::Min => vk::BlendOp::MIN,
            BlendOp::Max => vk::BlendOp::MAX,
        }
    }
}

impl From<ColorWriteMask> for vk::ColorComponentFlags {
    fn from(mask: ColorWriteMask) -> Self {
        let mut flags = vk::ColorComponentFlags::empty();

        if mask.contains(ColorWriteMask::R) {
            flags |= vk::ColorComponentFlags::R;
        }
        if mask.contains(ColorWriteMask::G) {
            flags |= vk::ColorComponentFlags::G;
        }
        if mask.contains(ColorWriteMask::B) {
            flags |= vk::ColorComponentFlags::B;
        }
        if mask.contains(ColorWriteMask::A) {
            flags |= vk::ColorComponentFlags::A;
        }

        flags
    }
}

impl From<CompareOp> for vk::CompareOp {
    fn from(op: CompareOp) -> Self {
        match op {
            CompareOp::Never => vk::CompareOp::NEVER,
            CompareOp::Less => vk::CompareOp::LESS,
            CompareOp::Equal => vk::CompareOp::EQUAL,
            CompareOp::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk::CompareOp::GREATER,
            CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareOp::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareOp::Always => vk::CompareOp::ALWAYS,
        }
    }
}

impl From<SamplerAddressMode> for vk::SamplerAddressMode {
    fn from(sampler_address_mode: SamplerAddressMode) -> Self {
        match sampler_address_mode {
            SamplerAddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
            SamplerAddressMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
            SamplerAddressMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            SamplerAddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        }
    }
}

impl From<Filter> for vk::Filter {
    fn from(filter: Filter) -> Self {
        match filter {
            Filter::Nearest => vk::Filter::NEAREST,
            Filter::Linear => vk::Filter::LINEAR,
        }
    }
}

impl From<ShaderStage> for vk::ShaderStageFlags {
    fn from(stage: ShaderStage) -> Self {
        let mut flags = vk::ShaderStageFlags::empty();
        if stage.contains(ShaderStage::VERTEX) {
            flags |= vk::ShaderStageFlags::VERTEX;
        }
        if stage.contains(ShaderStage::FRAGMENT) {
            flags |= vk::ShaderStageFlags::FRAGMENT;
        }
        if stage.contains(ShaderStage::COMPUTE) {
            flags |= vk::ShaderStageFlags::COMPUTE;
        }
        flags
    }
}
