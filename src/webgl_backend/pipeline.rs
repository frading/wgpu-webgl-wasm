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
        })
    }
}
