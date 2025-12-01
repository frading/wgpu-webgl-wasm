//! Shader module wrapper

use wasm_bindgen::prelude::*;
use std::sync::atomic::Ordering;
use super::device::WDevice;
use super::stats::SHADER_MODULE_COUNT;

/// WebGPU Shader Module wrapper
#[wasm_bindgen]
pub struct WShaderModule {
    pub(crate) inner: wgpu::ShaderModule,
}

impl WShaderModule {
    pub(crate) fn inner(&self) -> &wgpu::ShaderModule {
        &self.inner
    }
}

impl Drop for WShaderModule {
    fn drop(&mut self) {
        SHADER_MODULE_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Create a shader module from WGSL source
///
/// Unlike the old implementation that manually transpiled to GLSL,
/// wgpu handles this internally via Naga.
#[wasm_bindgen(js_name = createShaderModule)]
pub fn create_shader_module(
    device: &WDevice,
    wgsl_source: &str,
    _vertex_entry: &str,
    _fragment_entry: &str,
) -> Result<WShaderModule, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let module = state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(wgsl_source.into()),
    });

    log::debug!("Created shader module");

    SHADER_MODULE_COUNT.fetch_add(1, Ordering::Relaxed);

    Ok(WShaderModule { inner: module })
}

/// Transpile WGSL to GLSL (for debugging purposes)
/// This uses Naga directly, same as the old implementation
#[wasm_bindgen(js_name = transpileWgslToGlsl)]
pub fn transpile_wgsl_to_glsl(
    wgsl_source: &str,
    stage: u32, // 0 = vertex, 1 = fragment
    entry_point: &str,
) -> Result<String, JsValue> {
    use wgpu::naga;
    use naga::back::glsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    let naga_stage = match stage {
        0 => naga::ShaderStage::Vertex,
        1 => naga::ShaderStage::Fragment,
        _ => return Err(JsValue::from_str("Invalid shader stage")),
    };

    // Parse WGSL
    let module = naga::front::wgsl::parse_str(wgsl_source)
        .map_err(|e| JsValue::from_str(&format!("WGSL parse error: {:?}", e)))?;

    // Validate
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let info = validator
        .validate(&module)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {:?}", e)))?;

    // Transpile to GLSL ES 300
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        writer_flags: glsl::WriterFlags::ADJUST_COORDINATE_SPACE,
        ..Default::default()
    };

    let pipeline_options = glsl::PipelineOptions {
        shader_stage: naga_stage,
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
    .map_err(|e| JsValue::from_str(&format!("Writer creation error: {:?}", e)))?;

    writer
        .write()
        .map_err(|e| JsValue::from_str(&format!("Write error: {:?}", e)))?;

    Ok(output)
}
