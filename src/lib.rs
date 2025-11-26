//! Minimal proof-of-concept: WGPU WebGL2 backend exposed to WASM
//!
//! This module demonstrates how to use wgpu's GLES backend (which becomes WebGL2 on wasm32)
//! and expose it via wasm-bindgen for use from JavaScript.

use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

// Initialize console logging for debugging
#[wasm_bindgen(start)]
pub fn init() {
    console_log::init_with_level(log::Level::Info).ok();
    log::info!("wgpu-webgl-wasm initialized");
}

/// A simple test function to verify the WASM module loads correctly
#[wasm_bindgen]
pub fn test_wasm() -> String {
    "wgpu-webgl-wasm is working!".to_string()
}

/// Get the WebGL2 context from a canvas
/// This is a helper to demonstrate we can interact with WebGL2 from this WASM module
#[wasm_bindgen]
pub fn get_webgl2_context(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
    let context = canvas
        .get_context("webgl2")?
        .ok_or_else(|| JsValue::from_str("Failed to get WebGL2 context"))?
        .dyn_into::<WebGl2RenderingContext>()?;

    log::info!("WebGL2 context obtained successfully");
    Ok(context)
}

/// Demonstrates that naga can transpile WGSL to GLSL
/// This is the core functionality needed for WebGL2 support
#[wasm_bindgen]
pub fn transpile_wgsl_to_glsl(wgsl_source: &str) -> Result<String, JsValue> {
    use naga::back::glsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    // Parse WGSL
    let module = naga::front::wgsl::parse_str(wgsl_source)
        .map_err(|e| JsValue::from_str(&format!("WGSL parse error: {:?}", e)))?;

    // Validate
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let info = validator
        .validate(&module)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {:?}", e)))?;

    // Transpile to GLSL ES 300 (WebGL2)
    let options = glsl::Options {
        version: glsl::Version::Embedded { version: 300, is_webgl: true },
        ..Default::default()
    };

    let pipeline_options = glsl::PipelineOptions {
        shader_stage: naga::ShaderStage::Vertex,
        entry_point: "main".to_string(),
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
    .map_err(|e| JsValue::from_str(&format!("GLSL writer creation error: {:?}", e)))?;

    writer
        .write()
        .map_err(|e| JsValue::from_str(&format!("GLSL write error: {:?}", e)))?;

    log::info!("Successfully transpiled WGSL to GLSL ES 300");
    Ok(output)
}
