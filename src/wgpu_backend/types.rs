//! Common types and enums

use wasm_bindgen::prelude::*;

/// Primitive topology
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WPrimitiveTopology {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    TriangleList = 3,
    TriangleStrip = 4,
}

impl WPrimitiveTopology {
    pub(crate) fn to_wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Self::PointList => wgpu::PrimitiveTopology::PointList,
            Self::LineList => wgpu::PrimitiveTopology::LineList,
            Self::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            Self::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Self::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        }
    }
}

/// Vertex format
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WVertexFormat {
    Uint8x2 = 0,
    Uint8x4 = 1,
    Sint8x2 = 2,
    Sint8x4 = 3,
    Unorm8x2 = 4,
    Unorm8x4 = 5,
    Snorm8x2 = 6,
    Snorm8x4 = 7,
    Uint16x2 = 8,
    Uint16x4 = 9,
    Sint16x2 = 10,
    Sint16x4 = 11,
    Unorm16x2 = 12,
    Unorm16x4 = 13,
    Snorm16x2 = 14,
    Snorm16x4 = 15,
    Float16x2 = 16,
    Float16x4 = 17,
    Float32 = 18,
    Float32x2 = 19,
    Float32x3 = 20,
    Float32x4 = 21,
    Uint32 = 22,
    Uint32x2 = 23,
    Uint32x3 = 24,
    Uint32x4 = 25,
    Sint32 = 26,
    Sint32x2 = 27,
    Sint32x3 = 28,
    Sint32x4 = 29,
}

impl WVertexFormat {
    pub(crate) fn to_wgpu(self) -> wgpu::VertexFormat {
        match self {
            Self::Uint8x2 => wgpu::VertexFormat::Uint8x2,
            Self::Uint8x4 => wgpu::VertexFormat::Uint8x4,
            Self::Sint8x2 => wgpu::VertexFormat::Sint8x2,
            Self::Sint8x4 => wgpu::VertexFormat::Sint8x4,
            Self::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            Self::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
            Self::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            Self::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            Self::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            Self::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            Self::Sint16x2 => wgpu::VertexFormat::Sint16x2,
            Self::Sint16x4 => wgpu::VertexFormat::Sint16x4,
            Self::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            Self::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            Self::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
            Self::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
            Self::Float16x2 => wgpu::VertexFormat::Float16x2,
            Self::Float16x4 => wgpu::VertexFormat::Float16x4,
            Self::Float32 => wgpu::VertexFormat::Float32,
            Self::Float32x2 => wgpu::VertexFormat::Float32x2,
            Self::Float32x3 => wgpu::VertexFormat::Float32x3,
            Self::Float32x4 => wgpu::VertexFormat::Float32x4,
            Self::Uint32 => wgpu::VertexFormat::Uint32,
            Self::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            Self::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            Self::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            Self::Sint32 => wgpu::VertexFormat::Sint32,
            Self::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            Self::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            Self::Sint32x4 => wgpu::VertexFormat::Sint32x4,
        }
    }
}

/// Load operation for render pass
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WLoadOp {
    Clear = 0,
    Load = 1,
}

/// Store operation for render pass
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WStoreOp {
    Store = 0,
    Discard = 1,
}

/// Index format
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WIndexFormat {
    Uint16 = 0,
    Uint32 = 1,
}

impl WIndexFormat {
    pub(crate) fn to_wgpu(self) -> wgpu::IndexFormat {
        match self {
            Self::Uint16 => wgpu::IndexFormat::Uint16,
            Self::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

/// Cull mode
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WCullMode {
    None = 0,
    Front = 1,
    Back = 2,
}

impl WCullMode {
    pub(crate) fn to_wgpu(self) -> Option<wgpu::Face> {
        match self {
            Self::None => None,
            Self::Front => Some(wgpu::Face::Front),
            Self::Back => Some(wgpu::Face::Back),
        }
    }
}

/// Front face winding
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WFrontFace {
    Ccw = 0,
    Cw = 1,
}

impl WFrontFace {
    pub(crate) fn to_wgpu(self) -> wgpu::FrontFace {
        match self {
            Self::Ccw => wgpu::FrontFace::Ccw,
            Self::Cw => wgpu::FrontFace::Cw,
        }
    }
}

/// Blend factor
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WBlendFactor {
    Zero = 0,
    One = 1,
    Src = 2,
    OneMinusSrc = 3,
    SrcAlpha = 4,
    OneMinusSrcAlpha = 5,
    Dst = 6,
    OneMinusDst = 7,
    DstAlpha = 8,
    OneMinusDstAlpha = 9,
    SrcAlphaSaturated = 10,
    Constant = 11,
    OneMinusConstant = 12,
}

impl WBlendFactor {
    pub(crate) fn to_wgpu(self) -> wgpu::BlendFactor {
        match self {
            Self::Zero => wgpu::BlendFactor::Zero,
            Self::One => wgpu::BlendFactor::One,
            Self::Src => wgpu::BlendFactor::Src,
            Self::OneMinusSrc => wgpu::BlendFactor::OneMinusSrc,
            Self::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            Self::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            Self::Dst => wgpu::BlendFactor::Dst,
            Self::OneMinusDst => wgpu::BlendFactor::OneMinusDst,
            Self::DstAlpha => wgpu::BlendFactor::DstAlpha,
            Self::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            Self::SrcAlphaSaturated => wgpu::BlendFactor::SrcAlphaSaturated,
            Self::Constant => wgpu::BlendFactor::Constant,
            Self::OneMinusConstant => wgpu::BlendFactor::OneMinusConstant,
        }
    }
}

/// Blend operation
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WBlendOperation {
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
}

impl WBlendOperation {
    pub(crate) fn to_wgpu(self) -> wgpu::BlendOperation {
        match self {
            Self::Add => wgpu::BlendOperation::Add,
            Self::Subtract => wgpu::BlendOperation::Subtract,
            Self::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
            Self::Min => wgpu::BlendOperation::Min,
            Self::Max => wgpu::BlendOperation::Max,
        }
    }
}

/// Shader stage flags
pub mod shader_stage {
    pub const VERTEX: u32 = 1;
    pub const FRAGMENT: u32 = 2;
    pub const COMPUTE: u32 = 4;
}

/// Texture usage flags
pub mod texture_usage {
    pub const COPY_SRC: u32 = 1;
    pub const COPY_DST: u32 = 2;
    pub const TEXTURE_BINDING: u32 = 4;
    pub const STORAGE_BINDING: u32 = 8;
    pub const RENDER_ATTACHMENT: u32 = 16;
}

/// Shader stage
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WShaderStage {
    Vertex = 0,
    Fragment = 1,
    Compute = 2,
}

/// Vertex step mode
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WVertexStepMode {
    /// Attribute advances per vertex (default)
    Vertex = 0,
    /// Attribute advances per instance
    Instance = 1,
}

impl WVertexStepMode {
    pub(crate) fn to_wgpu(self) -> wgpu::VertexStepMode {
        match self {
            Self::Vertex => wgpu::VertexStepMode::Vertex,
            Self::Instance => wgpu::VertexStepMode::Instance,
        }
    }
}

/// Blend component - describes how to blend either color or alpha
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WBlendComponent {
    pub operation: WBlendOperation,
    pub src_factor: WBlendFactor,
    pub dst_factor: WBlendFactor,
}

#[wasm_bindgen]
impl WBlendComponent {
    #[wasm_bindgen(constructor)]
    pub fn new(operation: WBlendOperation, src_factor: WBlendFactor, dst_factor: WBlendFactor) -> Self {
        Self { operation, src_factor, dst_factor }
    }
}

/// Blend state - describes blending for a color attachment
#[wasm_bindgen]
pub struct WBlendState {
    pub color: WBlendComponent,
    pub alpha: WBlendComponent,
}

#[wasm_bindgen]
impl WBlendState {
    #[wasm_bindgen(constructor)]
    pub fn new(
        color_op: WBlendOperation,
        color_src: WBlendFactor,
        color_dst: WBlendFactor,
        alpha_op: WBlendOperation,
        alpha_src: WBlendFactor,
        alpha_dst: WBlendFactor,
    ) -> Self {
        Self {
            color: WBlendComponent::new(color_op, color_src, color_dst),
            alpha: WBlendComponent::new(alpha_op, alpha_src, alpha_dst),
        }
    }

    /// Check if blending is enabled (not just overwrite)
    pub fn is_enabled(&self) -> bool {
        // Blend is "disabled" if it's effectively a simple overwrite
        !(self.color.operation == WBlendOperation::Add
            && self.color.src_factor == WBlendFactor::One
            && self.color.dst_factor == WBlendFactor::Zero
            && self.alpha.operation == WBlendOperation::Add
            && self.alpha.src_factor == WBlendFactor::One
            && self.alpha.dst_factor == WBlendFactor::Zero)
    }
}

/// Vertex attribute description for pipeline creation
#[wasm_bindgen]
pub struct WVertexAttribute {
    /// Shader location (e.g., 0 for @location(0))
    pub location: u32,
    /// Byte offset within the vertex
    pub offset: u32,
    /// Format of the attribute
    pub format: WVertexFormat,
}

#[wasm_bindgen]
impl WVertexAttribute {
    #[wasm_bindgen(constructor)]
    pub fn new(location: u32, offset: u32, format: WVertexFormat) -> Self {
        Self { location, offset, format }
    }
}

/// Vertex buffer layout description
#[wasm_bindgen]
pub struct WVertexBufferLayout {
    /// Stride in bytes between consecutive vertices
    pub stride: u32,
    pub(crate) attributes: Vec<WVertexAttribute>,
}

#[wasm_bindgen]
impl WVertexBufferLayout {
    #[wasm_bindgen(constructor)]
    pub fn new(stride: u32) -> Self {
        Self { stride, attributes: Vec::new() }
    }

    /// Add an attribute to this buffer layout
    #[wasm_bindgen(js_name = addAttribute)]
    pub fn add_attribute(&mut self, location: u32, offset: u32, format: WVertexFormat) {
        self.attributes.push(WVertexAttribute::new(location, offset, format));
    }
}

/// Get buffer usage constants (for JS access)
#[wasm_bindgen(js_name = getBufferUsage)]
pub fn get_buffer_usage() -> JsValue {
    use super::buffer::buffer_usage;
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"MAP_READ".into(), &buffer_usage::MAP_READ.into()).unwrap();
    js_sys::Reflect::set(&obj, &"MAP_WRITE".into(), &buffer_usage::MAP_WRITE.into()).unwrap();
    js_sys::Reflect::set(&obj, &"COPY_SRC".into(), &buffer_usage::COPY_SRC.into()).unwrap();
    js_sys::Reflect::set(&obj, &"COPY_DST".into(), &buffer_usage::COPY_DST.into()).unwrap();
    js_sys::Reflect::set(&obj, &"INDEX".into(), &buffer_usage::INDEX.into()).unwrap();
    js_sys::Reflect::set(&obj, &"VERTEX".into(), &buffer_usage::VERTEX.into()).unwrap();
    js_sys::Reflect::set(&obj, &"UNIFORM".into(), &buffer_usage::UNIFORM.into()).unwrap();
    js_sys::Reflect::set(&obj, &"STORAGE".into(), &buffer_usage::STORAGE.into()).unwrap();
    obj.into()
}
