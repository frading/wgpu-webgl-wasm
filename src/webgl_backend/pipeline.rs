//! Render pipeline creation and management

use super::device::GlContextRef;
use super::shader::WShaderModule;
use super::types::{WPrimitiveTopology, WVertexFormat};
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Stored vertex attribute for later configuration
#[derive(Clone)]
pub struct StoredVertexAttribute {
    pub location: u32,
    pub offset: u32,
    pub format: WVertexFormat,
}

/// Stored vertex buffer layout for later configuration
#[derive(Clone)]
pub struct StoredVertexBufferLayout {
    pub stride: u32,
    pub attributes: Vec<StoredVertexAttribute>,
}

/// Render pipeline - equivalent to GPURenderPipeline
#[wasm_bindgen]
pub struct WRenderPipeline {
    context: GlContextRef,
    pub(crate) program: glow::Program,
    pub(crate) vao: glow::VertexArray,
    pub(crate) topology: WPrimitiveTopology,
    /// Stored vertex layout for configuring attributes when buffer is bound
    pub(crate) vertex_layout: Option<StoredVertexBufferLayout>,
    // Rasterization state
    pub(crate) cull_mode: WCullMode,
    pub(crate) front_face: WFrontFace,
    // Depth state
    pub(crate) depth_test_enabled: bool,
    pub(crate) depth_write_enabled: bool,
    pub(crate) depth_compare: WCompareFunction,
}

impl Drop for WRenderPipeline {
    fn drop(&mut self) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.delete_program(self.program);
            ctx.gl.delete_vertex_array(self.vao);
        }
        log::debug!("Render pipeline destroyed");
    }
}

impl WRenderPipeline {
    pub fn context(&self) -> GlContextRef {
        self.context.clone()
    }
}

#[wasm_bindgen]
impl WRenderPipeline {
    /// Get bind group layout at the specified index
    /// In WebGL, this returns an empty layout since we don't have explicit bind group layouts
    /// tied to pipelines. The layout is inferred from shader reflection at bind time.
    #[wasm_bindgen(js_name = getBindGroupLayout)]
    pub fn get_bind_group_layout(&self, _index: u32) -> super::bind_group::WBindGroupLayout {
        // In WebGL, bind group layouts are not explicitly tied to pipelines
        // We return an empty layout that can be used for compatibility
        log::debug!("getBindGroupLayout called for index {}", _index);
        super::bind_group::WBindGroupLayout {
            entries: Vec::new(),
        }
    }
}

/// Create a render pipeline (simple version without vertex attributes)
/// This links shaders into a program and sets up the vertex array object
#[wasm_bindgen(js_name = createRenderPipeline)]
pub fn create_render_pipeline(
    device: &super::WDevice,
    shader_module: &WShaderModule,
    topology: WPrimitiveTopology,
) -> Result<WRenderPipeline, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        // Create program and link shaders
        let program = ctx
            .gl
            .create_program()
            .map_err(|e| JsValue::from_str(&format!("Failed to create program: {}", e)))?;

        if let Some(vs) = shader_module.vertex_shader {
            ctx.gl.attach_shader(program, vs);
        }
        if let Some(fs) = shader_module.fragment_shader {
            ctx.gl.attach_shader(program, fs);
        }

        ctx.gl.link_program(program);

        if !ctx.gl.get_program_link_status(program) {
            let log = ctx.gl.get_program_info_log(program);
            ctx.gl.delete_program(program);
            return Err(JsValue::from_str(&format!(
                "Program linking failed: {}",
                log
            )));
        }

        // Create VAO (required for WebGL2)
        let vao = ctx
            .gl
            .create_vertex_array()
            .map_err(|e| JsValue::from_str(&format!("Failed to create VAO: {}", e)))?;

        log::info!("Render pipeline created successfully");

        Ok(WRenderPipeline {
            context: context.clone(),
            program,
            vao,
            topology,
            vertex_layout: None,
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
        })
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
    /// Attributes in this buffer
    attributes: Vec<WVertexAttribute>,
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
        self.attributes.push(WVertexAttribute { location, offset, format });
    }
}

/// Cull mode for rasterization
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WCullMode {
    None = 0,
    Front = 1,
    Back = 2,
}

impl WCullMode {
    pub fn to_gl(self) -> Option<u32> {
        match self {
            WCullMode::None => None,
            WCullMode::Front => Some(glow::FRONT),
            WCullMode::Back => Some(glow::BACK),
        }
    }
}

/// Front face winding order
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WFrontFace {
    Ccw = 0,
    Cw = 1,
}

impl WFrontFace {
    pub fn to_gl(self) -> u32 {
        match self {
            WFrontFace::Ccw => glow::CCW,
            WFrontFace::Cw => glow::CW,
        }
    }
}

/// Compare function for depth/stencil
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WCompareFunction {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

impl WCompareFunction {
    pub fn to_gl(self) -> u32 {
        match self {
            WCompareFunction::Never => glow::NEVER,
            WCompareFunction::Less => glow::LESS,
            WCompareFunction::Equal => glow::EQUAL,
            WCompareFunction::LessEqual => glow::LEQUAL,
            WCompareFunction::Greater => glow::GREATER,
            WCompareFunction::NotEqual => glow::NOTEQUAL,
            WCompareFunction::GreaterEqual => glow::GEQUAL,
            WCompareFunction::Always => glow::ALWAYS,
        }
    }
}

/// Extended render pipeline with more state
#[wasm_bindgen]
pub struct WRenderPipelineDescriptor {
    // Primitive state
    topology: WPrimitiveTopology,
    cull_mode: WCullMode,
    front_face: WFrontFace,
    // Depth state
    depth_test_enabled: bool,
    depth_write_enabled: bool,
    depth_compare: WCompareFunction,
    // Vertex layouts (up to 4)
    vertex_layouts: Vec<StoredVertexBufferLayout>,
}

#[wasm_bindgen]
impl WRenderPipelineDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new(topology: WPrimitiveTopology) -> Self {
        Self {
            topology,
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
            vertex_layouts: Vec::new(),
        }
    }

    #[wasm_bindgen(js_name = setCullMode)]
    pub fn set_cull_mode(&mut self, cull_mode: WCullMode) {
        self.cull_mode = cull_mode;
    }

    #[wasm_bindgen(js_name = setFrontFace)]
    pub fn set_front_face(&mut self, front_face: WFrontFace) {
        self.front_face = front_face;
    }

    #[wasm_bindgen(js_name = setDepthTest)]
    pub fn set_depth_test(&mut self, enabled: bool, write_enabled: bool, compare: WCompareFunction) {
        self.depth_test_enabled = enabled;
        self.depth_write_enabled = write_enabled;
        self.depth_compare = compare;
    }

    #[wasm_bindgen(js_name = addVertexBufferLayout)]
    pub fn add_vertex_buffer_layout(&mut self, stride: u32) -> usize {
        let index = self.vertex_layouts.len();
        self.vertex_layouts.push(StoredVertexBufferLayout {
            stride,
            attributes: Vec::new(),
        });
        index
    }

    #[wasm_bindgen(js_name = addVertexAttribute)]
    pub fn add_vertex_attribute(&mut self, buffer_index: usize, location: u32, offset: u32, format: WVertexFormat) {
        if buffer_index < self.vertex_layouts.len() {
            self.vertex_layouts[buffer_index].attributes.push(StoredVertexAttribute {
                location,
                offset,
                format,
            });
        }
    }
}

/// Create a render pipeline with full descriptor
#[wasm_bindgen(js_name = createRenderPipelineFromDescriptor)]
pub fn create_render_pipeline_from_descriptor(
    device: &super::WDevice,
    shader_module: &WShaderModule,
    descriptor: &WRenderPipelineDescriptor,
) -> Result<WRenderPipeline, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        // Create program and link shaders
        let program = ctx
            .gl
            .create_program()
            .map_err(|e| JsValue::from_str(&format!("Failed to create program: {}", e)))?;

        if let Some(vs) = shader_module.vertex_shader {
            ctx.gl.attach_shader(program, vs);
        }
        if let Some(fs) = shader_module.fragment_shader {
            ctx.gl.attach_shader(program, fs);
        }

        ctx.gl.link_program(program);

        if !ctx.gl.get_program_link_status(program) {
            let log = ctx.gl.get_program_info_log(program);
            ctx.gl.delete_program(program);
            return Err(JsValue::from_str(&format!(
                "Program linking failed: {}",
                log
            )));
        }

        // Create VAO
        let vao = ctx
            .gl
            .create_vertex_array()
            .map_err(|e| JsValue::from_str(&format!("Failed to create VAO: {}", e)))?;

        // Get the first vertex layout if any
        let vertex_layout = descriptor.vertex_layouts.first().cloned();

        log::info!("Render pipeline created with {} vertex buffer layouts", descriptor.vertex_layouts.len());

        Ok(WRenderPipeline {
            context: context.clone(),
            program,
            vao,
            topology: descriptor.topology,
            vertex_layout,
            cull_mode: descriptor.cull_mode,
            front_face: descriptor.front_face,
            depth_test_enabled: descriptor.depth_test_enabled,
            depth_write_enabled: descriptor.depth_write_enabled,
            depth_compare: descriptor.depth_compare,
        })
    }
}

/// Create a render pipeline with vertex buffer layout
/// This version allows specifying vertex attributes from a buffer
#[wasm_bindgen(js_name = createRenderPipelineWithLayout)]
pub fn create_render_pipeline_with_layout(
    device: &super::WDevice,
    shader_module: &WShaderModule,
    topology: WPrimitiveTopology,
    vertex_layout: &WVertexBufferLayout,
) -> Result<WRenderPipeline, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        // Create program and link shaders
        let program = ctx
            .gl
            .create_program()
            .map_err(|e| JsValue::from_str(&format!("Failed to create program: {}", e)))?;

        if let Some(vs) = shader_module.vertex_shader {
            ctx.gl.attach_shader(program, vs);
        }
        if let Some(fs) = shader_module.fragment_shader {
            ctx.gl.attach_shader(program, fs);
        }

        ctx.gl.link_program(program);

        if !ctx.gl.get_program_link_status(program) {
            let log = ctx.gl.get_program_info_log(program);
            ctx.gl.delete_program(program);
            return Err(JsValue::from_str(&format!(
                "Program linking failed: {}",
                log
            )));
        }

        // Create VAO (required for WebGL2)
        let vao = ctx
            .gl
            .create_vertex_array()
            .map_err(|e| JsValue::from_str(&format!("Failed to create VAO: {}", e)))?;

        // Store the vertex layout for later use when setVertexBuffer is called
        // In WebGL, glVertexAttribPointer captures the currently bound buffer,
        // so we can't configure attributes until the buffer is bound.
        let stored_layout = StoredVertexBufferLayout {
            stride: vertex_layout.stride,
            attributes: vertex_layout.attributes.iter().map(|attr| {
                StoredVertexAttribute {
                    location: attr.location,
                    offset: attr.offset,
                    format: attr.format,
                }
            }).collect(),
        };

        log::info!("Render pipeline with vertex layout created successfully");

        Ok(WRenderPipeline {
            context: context.clone(),
            program,
            vao,
            topology,
            vertex_layout: Some(stored_layout),
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
        })
    }
}
