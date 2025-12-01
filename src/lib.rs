//! WGPU WebGL2 WASM Bridge
//!
//! This module provides a WebGPU-like API using wgpu's WebGL backend internally.
//! It mirrors the WebGPU API so that JS can use the same calling pattern
//! regardless of whether native WebGPU or this WebGL2 fallback is used.
//!
//! By using wgpu internally (instead of raw WebGL calls), we get:
//! - Proper shadow mapping support (depth textures, comparison samplers)
//! - Texture arrays
//! - All coordinate space conversions handled correctly
//! - Battle-tested code from Firefox/Servo

use wasm_bindgen::prelude::*;

mod wgpu_backend;

pub use wgpu_backend::*;

// Initialize console logging for debugging
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in the browser
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Use Warn level to filter out verbose naga/wgpu internal logs
    // (Resolving, wgsl automatic_conversion_consensus, var GlobalVariable, Naga generated, etc.)
    console_log::init_with_level(log::Level::Warn).ok();
    log::info!("wgpu-webgl-wasm initialized (wgpu backend v2)");
}

/// A simple test function to verify the WASM module loads correctly
#[wasm_bindgen]
pub fn test_wasm() -> String {
    "wgpu-webgl-wasm is working! (wgpu backend)".to_string()
}

/// Returns true because this is the WebGL2 fallback backend
/// Use this to detect if we're running on WebGL2 vs native WebGPU
#[wasm_bindgen(js_name = isWebGL2Backend)]
pub fn is_webgl2_backend() -> bool {
    true
}

/// Returns information about WebGL2 limitations
#[wasm_bindgen(js_name = getBackendLimitations)]
pub fn get_backend_limitations() -> JsValue {
    let limitations = js_sys::Object::new();

    // Depth texture arrays are not well-supported on WebGL2
    let _ = js_sys::Reflect::set(&limitations, &"depthTextureArrays".into(), &false.into());

    // Storage buffers have limited support
    let _ = js_sys::Reflect::set(&limitations, &"storageBuffers".into(), &false.into());

    // Compute shaders not available
    let _ = js_sys::Reflect::set(&limitations, &"computeShaders".into(), &false.into());

    limitations.into()
}

/// Returns WASM memory usage statistics
#[wasm_bindgen(js_name = getMemoryStats)]
pub fn get_memory_stats() -> JsValue {
    let stats = js_sys::Object::new();

    // Get WASM memory size
    let memory = wasm_bindgen::memory();
    let buffer = memory.dyn_ref::<js_sys::WebAssembly::Memory>().unwrap().buffer();
    let wasm_memory_bytes = buffer.dyn_ref::<js_sys::ArrayBuffer>().unwrap().byte_length();

    let _ = js_sys::Reflect::set(&stats, &"wasmMemoryBytes".into(), &wasm_memory_bytes.into());
    let _ = js_sys::Reflect::set(&stats, &"wasmMemoryMB".into(), &((wasm_memory_bytes as f64) / (1024.0 * 1024.0)).into());

    stats.into()
}
