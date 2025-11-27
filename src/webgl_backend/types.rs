//! Common types and enums mirroring WebGPU types

use wasm_bindgen::prelude::*;

/// Primitive topology for rendering
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
    pub fn to_gl(self) -> u32 {
        match self {
            WPrimitiveTopology::PointList => glow::POINTS,
            WPrimitiveTopology::LineList => glow::LINES,
            WPrimitiveTopology::LineStrip => glow::LINE_STRIP,
            WPrimitiveTopology::TriangleList => glow::TRIANGLES,
            WPrimitiveTopology::TriangleStrip => glow::TRIANGLE_STRIP,
        }
    }
}

/// Vertex format types supported by WebGL2
/// These map to glVertexAttribPointer/glVertexAttribIPointer parameters
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WVertexFormat {
    // 8-bit formats
    Uint8x2 = 0,
    Uint8x4 = 1,
    Sint8x2 = 2,
    Sint8x4 = 3,
    Unorm8x2 = 4,
    Unorm8x4 = 5,
    Snorm8x2 = 6,
    Snorm8x4 = 7,
    // 16-bit formats
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
    // 32-bit formats
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
    pub fn size(self) -> u32 {
        match self {
            WVertexFormat::Uint8x2 | WVertexFormat::Sint8x2 | WVertexFormat::Unorm8x2 | WVertexFormat::Snorm8x2 => 2,
            WVertexFormat::Uint8x4 | WVertexFormat::Sint8x4 | WVertexFormat::Unorm8x4 | WVertexFormat::Snorm8x4 => 4,
            WVertexFormat::Uint16x2 | WVertexFormat::Sint16x2 | WVertexFormat::Unorm16x2 | WVertexFormat::Snorm16x2 | WVertexFormat::Float16x2 => 4,
            WVertexFormat::Uint16x4 | WVertexFormat::Sint16x4 | WVertexFormat::Unorm16x4 | WVertexFormat::Snorm16x4 | WVertexFormat::Float16x4 => 8,
            WVertexFormat::Float32 | WVertexFormat::Uint32 | WVertexFormat::Sint32 => 4,
            WVertexFormat::Float32x2 | WVertexFormat::Uint32x2 | WVertexFormat::Sint32x2 => 8,
            WVertexFormat::Float32x3 | WVertexFormat::Uint32x3 | WVertexFormat::Sint32x3 => 12,
            WVertexFormat::Float32x4 | WVertexFormat::Uint32x4 | WVertexFormat::Sint32x4 => 16,
        }
    }

    pub fn components(self) -> i32 {
        match self {
            WVertexFormat::Float32 | WVertexFormat::Uint32 | WVertexFormat::Sint32 => 1,
            WVertexFormat::Uint8x2 | WVertexFormat::Sint8x2 | WVertexFormat::Unorm8x2 | WVertexFormat::Snorm8x2 |
            WVertexFormat::Uint16x2 | WVertexFormat::Sint16x2 | WVertexFormat::Unorm16x2 | WVertexFormat::Snorm16x2 |
            WVertexFormat::Float16x2 | WVertexFormat::Float32x2 | WVertexFormat::Uint32x2 | WVertexFormat::Sint32x2 => 2,
            WVertexFormat::Float32x3 | WVertexFormat::Uint32x3 | WVertexFormat::Sint32x3 => 3,
            WVertexFormat::Uint8x4 | WVertexFormat::Sint8x4 | WVertexFormat::Unorm8x4 | WVertexFormat::Snorm8x4 |
            WVertexFormat::Uint16x4 | WVertexFormat::Sint16x4 | WVertexFormat::Unorm16x4 | WVertexFormat::Snorm16x4 |
            WVertexFormat::Float16x4 | WVertexFormat::Float32x4 | WVertexFormat::Uint32x4 | WVertexFormat::Sint32x4 => 4,
        }
    }

    pub fn gl_type(self) -> u32 {
        match self {
            WVertexFormat::Uint8x2 | WVertexFormat::Uint8x4 | WVertexFormat::Unorm8x2 | WVertexFormat::Unorm8x4 => glow::UNSIGNED_BYTE,
            WVertexFormat::Sint8x2 | WVertexFormat::Sint8x4 | WVertexFormat::Snorm8x2 | WVertexFormat::Snorm8x4 => glow::BYTE,
            WVertexFormat::Uint16x2 | WVertexFormat::Uint16x4 | WVertexFormat::Unorm16x2 | WVertexFormat::Unorm16x4 => glow::UNSIGNED_SHORT,
            WVertexFormat::Sint16x2 | WVertexFormat::Sint16x4 | WVertexFormat::Snorm16x2 | WVertexFormat::Snorm16x4 => glow::SHORT,
            WVertexFormat::Float16x2 | WVertexFormat::Float16x4 => glow::HALF_FLOAT,
            WVertexFormat::Float32 | WVertexFormat::Float32x2 | WVertexFormat::Float32x3 | WVertexFormat::Float32x4 => glow::FLOAT,
            WVertexFormat::Uint32 | WVertexFormat::Uint32x2 | WVertexFormat::Uint32x3 | WVertexFormat::Uint32x4 => glow::UNSIGNED_INT,
            WVertexFormat::Sint32 | WVertexFormat::Sint32x2 | WVertexFormat::Sint32x3 | WVertexFormat::Sint32x4 => glow::INT,
        }
    }

    /// Whether this format should be normalized when passed to glVertexAttribPointer
    pub fn normalized(self) -> bool {
        matches!(self,
            WVertexFormat::Unorm8x2 | WVertexFormat::Unorm8x4 |
            WVertexFormat::Snorm8x2 | WVertexFormat::Snorm8x4 |
            WVertexFormat::Unorm16x2 | WVertexFormat::Unorm16x4 |
            WVertexFormat::Snorm16x2 | WVertexFormat::Snorm16x4
        )
    }

    /// Whether this format requires glVertexAttribIPointer (integer formats)
    pub fn is_integer(self) -> bool {
        matches!(self,
            WVertexFormat::Uint8x2 | WVertexFormat::Uint8x4 |
            WVertexFormat::Sint8x2 | WVertexFormat::Sint8x4 |
            WVertexFormat::Uint16x2 | WVertexFormat::Uint16x4 |
            WVertexFormat::Sint16x2 | WVertexFormat::Sint16x4 |
            WVertexFormat::Uint32 | WVertexFormat::Uint32x2 | WVertexFormat::Uint32x3 | WVertexFormat::Uint32x4 |
            WVertexFormat::Sint32 | WVertexFormat::Sint32x2 | WVertexFormat::Sint32x3 | WVertexFormat::Sint32x4
        )
    }
}

/// Buffer usage flags - exposed as constants via JS
pub mod buffer_usage {
    pub const MAP_READ: u32 = 0x0001;
    pub const MAP_WRITE: u32 = 0x0002;
    pub const COPY_SRC: u32 = 0x0004;
    pub const COPY_DST: u32 = 0x0008;
    pub const INDEX: u32 = 0x0010;
    pub const VERTEX: u32 = 0x0020;
    pub const UNIFORM: u32 = 0x0040;
    pub const STORAGE: u32 = 0x0080;
    pub const INDIRECT: u32 = 0x0100;
    pub const QUERY_RESOLVE: u32 = 0x0200;
}

/// Get buffer usage constants (for JS access)
#[wasm_bindgen(js_name = getBufferUsage)]
pub fn get_buffer_usage() -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"MAP_READ".into(), &buffer_usage::MAP_READ.into()).unwrap();
    js_sys::Reflect::set(&obj, &"MAP_WRITE".into(), &buffer_usage::MAP_WRITE.into()).unwrap();
    js_sys::Reflect::set(&obj, &"COPY_SRC".into(), &buffer_usage::COPY_SRC.into()).unwrap();
    js_sys::Reflect::set(&obj, &"COPY_DST".into(), &buffer_usage::COPY_DST.into()).unwrap();
    js_sys::Reflect::set(&obj, &"INDEX".into(), &buffer_usage::INDEX.into()).unwrap();
    js_sys::Reflect::set(&obj, &"VERTEX".into(), &buffer_usage::VERTEX.into()).unwrap();
    js_sys::Reflect::set(&obj, &"UNIFORM".into(), &buffer_usage::UNIFORM.into()).unwrap();
    js_sys::Reflect::set(&obj, &"STORAGE".into(), &buffer_usage::STORAGE.into()).unwrap();
    js_sys::Reflect::set(&obj, &"INDIRECT".into(), &buffer_usage::INDIRECT.into()).unwrap();
    js_sys::Reflect::set(&obj, &"QUERY_RESOLVE".into(), &buffer_usage::QUERY_RESOLVE.into()).unwrap();
    obj.into()
}

/// Load operation for render pass attachments
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WLoadOp {
    Clear = 0,
    Load = 1,
}

/// Store operation for render pass attachments
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WStoreOp {
    Store = 0,
    Discard = 1,
}

/// Shader stage
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WShaderStage {
    Vertex = 0,
    Fragment = 1,
    Compute = 2,
}

impl WShaderStage {
    pub fn to_naga(self) -> naga::ShaderStage {
        match self {
            WShaderStage::Vertex => naga::ShaderStage::Vertex,
            WShaderStage::Fragment => naga::ShaderStage::Fragment,
            WShaderStage::Compute => naga::ShaderStage::Compute,
        }
    }
}

/// Blend factor - maps to WebGPU GPUBlendFactor
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WBlendFactor {
    #[default]
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
    pub fn to_gl(self) -> u32 {
        match self {
            WBlendFactor::Zero => glow::ZERO,
            WBlendFactor::One => glow::ONE,
            WBlendFactor::Src => glow::SRC_COLOR,
            WBlendFactor::OneMinusSrc => glow::ONE_MINUS_SRC_COLOR,
            WBlendFactor::SrcAlpha => glow::SRC_ALPHA,
            WBlendFactor::OneMinusSrcAlpha => glow::ONE_MINUS_SRC_ALPHA,
            WBlendFactor::Dst => glow::DST_COLOR,
            WBlendFactor::OneMinusDst => glow::ONE_MINUS_DST_COLOR,
            WBlendFactor::DstAlpha => glow::DST_ALPHA,
            WBlendFactor::OneMinusDstAlpha => glow::ONE_MINUS_DST_ALPHA,
            WBlendFactor::SrcAlphaSaturated => glow::SRC_ALPHA_SATURATE,
            WBlendFactor::Constant => glow::CONSTANT_COLOR,
            WBlendFactor::OneMinusConstant => glow::ONE_MINUS_CONSTANT_COLOR,
        }
    }
}

/// Blend operation - maps to WebGPU GPUBlendOperation
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WBlendOperation {
    #[default]
    Add = 0,
    Subtract = 1,
    ReverseSubtract = 2,
    Min = 3,
    Max = 4,
}

impl WBlendOperation {
    pub fn to_gl(self) -> u32 {
        match self {
            WBlendOperation::Add => glow::FUNC_ADD,
            WBlendOperation::Subtract => glow::FUNC_SUBTRACT,
            WBlendOperation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
            WBlendOperation::Min => glow::MIN,
            WBlendOperation::Max => glow::MAX,
        }
    }
}

/// Blend component - describes how to blend either color or alpha
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default)]
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
#[derive(Clone, Copy, Debug, Default)]
pub struct WBlendState {
    pub color: WBlendComponent,
    pub alpha: WBlendComponent,
}

#[wasm_bindgen]
impl WBlendState {
    #[wasm_bindgen(constructor)]
    pub fn new(
        color_op: WBlendOperation, color_src: WBlendFactor, color_dst: WBlendFactor,
        alpha_op: WBlendOperation, alpha_src: WBlendFactor, alpha_dst: WBlendFactor,
    ) -> Self {
        Self {
            color: WBlendComponent { operation: color_op, src_factor: color_src, dst_factor: color_dst },
            alpha: WBlendComponent { operation: alpha_op, src_factor: alpha_src, dst_factor: alpha_dst },
        }
    }

    /// Check if blending is enabled (not just overwrite)
    pub fn is_enabled(&self) -> bool {
        // Blending is effectively disabled if both src=One, dst=Zero
        !(self.color.src_factor == WBlendFactor::One && self.color.dst_factor == WBlendFactor::Zero &&
          self.alpha.src_factor == WBlendFactor::One && self.alpha.dst_factor == WBlendFactor::Zero)
    }
}
