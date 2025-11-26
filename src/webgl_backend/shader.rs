//! Shader module creation with WGSL to GLSL transpilation

use super::device::GlContextRef;
use super::types::WShaderStage;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Compiled shader module - equivalent to GPUShaderModule
#[wasm_bindgen]
pub struct WShaderModule {
    context: GlContextRef,
    /// Vertex shader (if present)
    pub(crate) vertex_shader: Option<glow::Shader>,
    /// Fragment shader (if present)
    pub(crate) fragment_shader: Option<glow::Shader>,
    /// Original WGSL source (for debugging)
    #[allow(dead_code)]
    wgsl_source: String,
}

impl Drop for WShaderModule {
    fn drop(&mut self) {
        let ctx = self.context.borrow();
        unsafe {
            if let Some(shader) = self.vertex_shader {
                ctx.gl.delete_shader(shader);
            }
            if let Some(shader) = self.fragment_shader {
                ctx.gl.delete_shader(shader);
            }
        }
        log::debug!("Shader module destroyed");
    }
}

/// Transpile WGSL to GLSL ES 300
pub fn transpile_wgsl_to_glsl(
    wgsl_source: &str,
    stage: naga::ShaderStage,
    entry_point: &str,
) -> Result<String, String> {
    use naga::back::glsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    // Parse WGSL
    let module = naga::front::wgsl::parse_str(wgsl_source)
        .map_err(|e| format!("WGSL parse error: {:?}", e))?;

    // Validate
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let info = validator
        .validate(&module)
        .map_err(|e| format!("Validation error: {:?}", e))?;

    // Transpile to GLSL ES 300 (WebGL2)
    // We keep ADJUST_COORDINATE_SPACE enabled because it does two things:
    // 1. Flips Y (for wgpu-hal's framebuffer blit - we don't need this)
    // 2. Remaps Z from WebGPU's [0,1] to OpenGL's [-1,1] (we DO need this for depth)
    // We'll post-process the GLSL to undo just the Y-flip.
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        // Keep default which includes ADJUST_COORDINATE_SPACE
        ..Default::default()
    };

    let pipeline_options = glsl::PipelineOptions {
        shader_stage: stage,
        entry_point: entry_point.to_string(),
        multiview: None,
    };

    let mut output = String::new();
    let mut writer = glsl::Writer::new(
        &mut output,
        &module,
        &info,
        &options,
        &pipeline_options,
        naga::proc::BoundsCheckPolicies::default(),
    )
    .map_err(|e| format!("GLSL writer creation error: {:?}", e))?;

    writer
        .write()
        .map_err(|e| format!("GLSL write error: {:?}", e))?;

    // Post-process vertex shaders to undo Y-flip while keeping Z remapping.
    // Naga generates: gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
    // We want:        gl_Position.z = gl_Position.z * 2.0 - gl_Position.w;
    // This keeps the depth remapping (WebGPU [0,1] -> OpenGL [-1,1]) but removes Y-flip.
    if stage == naga::ShaderStage::Vertex {
        output = undo_y_flip(&output);
    }

    Ok(output)
}

/// Undo the Y-flip in Naga's coordinate adjustment while keeping the Z remapping.
/// Naga generates: `gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);`
/// We replace with: `gl_Position.z = gl_Position.z * 2.0 - gl_Position.w;`
fn undo_y_flip(glsl_source: &str) -> String {
    glsl_source.replace(
        "gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);",
        "gl_Position.z = gl_Position.z * 2.0 - gl_Position.w;"
    )
}

/// Create a shader module from WGSL source
/// This transpiles WGSL to GLSL and compiles both vertex and fragment shaders
#[wasm_bindgen(js_name = createShaderModule)]
pub fn create_shader_module(
    device: &super::WDevice,
    wgsl_code: &str,
    vertex_entry_point: &str,
    fragment_entry_point: &str,
) -> Result<WShaderModule, JsValue> {
    let context = device.context();

    // Transpile vertex shader
    let vertex_glsl = transpile_wgsl_to_glsl(wgsl_code, naga::ShaderStage::Vertex, vertex_entry_point)
        .map_err(|e| JsValue::from_str(&e))?;

    // Transpile fragment shader
    let fragment_glsl = transpile_wgsl_to_glsl(wgsl_code, naga::ShaderStage::Fragment, fragment_entry_point)
        .map_err(|e| JsValue::from_str(&e))?;

    log::debug!("Vertex GLSL:\n{}", vertex_glsl);
    log::debug!("Fragment GLSL:\n{}", fragment_glsl);

    let ctx = context.borrow();

    unsafe {
        // Compile vertex shader
        let vertex_shader = ctx
            .gl
            .create_shader(glow::VERTEX_SHADER)
            .map_err(|e| JsValue::from_str(&format!("Failed to create vertex shader: {}", e)))?;
        ctx.gl.shader_source(vertex_shader, &vertex_glsl);
        ctx.gl.compile_shader(vertex_shader);

        if !ctx.gl.get_shader_compile_status(vertex_shader) {
            let log = ctx.gl.get_shader_info_log(vertex_shader);
            ctx.gl.delete_shader(vertex_shader);
            return Err(JsValue::from_str(&format!(
                "Vertex shader compilation failed: {}",
                log
            )));
        }

        // Compile fragment shader
        let fragment_shader = ctx
            .gl
            .create_shader(glow::FRAGMENT_SHADER)
            .map_err(|e| JsValue::from_str(&format!("Failed to create fragment shader: {}", e)))?;
        ctx.gl.shader_source(fragment_shader, &fragment_glsl);
        ctx.gl.compile_shader(fragment_shader);

        if !ctx.gl.get_shader_compile_status(fragment_shader) {
            let log = ctx.gl.get_shader_info_log(fragment_shader);
            ctx.gl.delete_shader(vertex_shader);
            ctx.gl.delete_shader(fragment_shader);
            return Err(JsValue::from_str(&format!(
                "Fragment shader compilation failed: {}",
                log
            )));
        }

        log::info!("Shader module created successfully");

        Ok(WShaderModule {
            context: context.clone(),
            vertex_shader: Some(vertex_shader),
            fragment_shader: Some(fragment_shader),
            wgsl_source: wgsl_code.to_string(),
        })
    }
}

/// Transpile WGSL to GLSL and return the result (for debugging)
#[wasm_bindgen(js_name = transpileWgslToGlsl)]
pub fn transpile_wgsl_to_glsl_js(
    wgsl_code: &str,
    stage: WShaderStage,
    entry_point: &str,
) -> Result<String, JsValue> {
    transpile_wgsl_to_glsl(wgsl_code, stage.to_naga(), entry_point)
        .map_err(|e| JsValue::from_str(&e))
}
