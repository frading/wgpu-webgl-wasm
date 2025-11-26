//! Test WGSL to GLSL transpilation

use naga::back::glsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

const TRIANGLE_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5)
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.2, 1.0);
}
"#;

fn transpile_wgsl_to_glsl(
    wgsl_source: &str,
    stage: naga::ShaderStage,
    entry_point: &str,
) -> Result<String, String> {
    let module = naga::front::wgsl::parse_str(wgsl_source)
        .map_err(|e| format!("WGSL parse error: {:?}", e))?;

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    let info = validator
        .validate(&module)
        .map_err(|e| format!("Validation error: {:?}", e))?;

    // Keep ADJUST_COORDINATE_SPACE enabled - it does Y-flip and Z remapping.
    // We'll post-process to undo just the Y-flip.
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
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

    Ok(output)
}

/// Undo the Y-flip in Naga's coordinate adjustment while keeping the Z remapping.
fn undo_y_flip(glsl_source: &str) -> String {
    glsl_source.replace(
        "gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);",
        "gl_Position.z = gl_Position.z * 2.0 - gl_Position.w;"
    )
}

#[test]
fn test_vertex_shader_raw_transpilation() {
    // Raw Naga output (with ADJUST_COORDINATE_SPACE)
    let glsl = transpile_wgsl_to_glsl(TRIANGLE_WGSL, naga::ShaderStage::Vertex, "vs_main")
        .expect("Failed to transpile vertex shader");

    println!("=== Raw Naga Vertex GLSL ===");
    println!("{}", glsl);

    assert!(glsl.contains("gl_Position"));
    assert!(glsl.contains("void main()"));
    // Should have both Y-flip and Z remapping
    assert!(glsl.contains("gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);"));
}

#[test]
fn test_vertex_shader_with_y_flip_removed() {
    // After post-processing to remove Y-flip but keep Z remapping
    let glsl = transpile_wgsl_to_glsl(TRIANGLE_WGSL, naga::ShaderStage::Vertex, "vs_main")
        .expect("Failed to transpile vertex shader");
    let processed = undo_y_flip(&glsl);

    println!("=== Processed Vertex GLSL (Y-flip removed) ===");
    println!("{}", processed);

    // Should NOT have Y-flip
    assert!(!processed.contains("-gl_Position.y"));
    // Should still have Z remapping
    assert!(processed.contains("gl_Position.z = gl_Position.z * 2.0 - gl_Position.w;"));
}

#[test]
fn test_fragment_shader_transpilation() {
    let glsl = transpile_wgsl_to_glsl(TRIANGLE_WGSL, naga::ShaderStage::Fragment, "fs_main")
        .expect("Failed to transpile fragment shader");

    println!("=== Generated Fragment GLSL ===");
    println!("{}", glsl);

    assert!(glsl.contains("void main()"));
}
