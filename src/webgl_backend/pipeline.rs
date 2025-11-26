//! Render pipeline creation and management

use super::device::GlContextRef;
use super::shader::WShaderModule;
use super::types::WPrimitiveTopology;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Render pipeline - equivalent to GPURenderPipeline
#[wasm_bindgen]
pub struct WRenderPipeline {
    context: GlContextRef,
    pub(crate) program: glow::Program,
    pub(crate) vao: glow::VertexArray,
    pub(crate) topology: WPrimitiveTopology,
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

/// Create a render pipeline
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
        })
    }
}
