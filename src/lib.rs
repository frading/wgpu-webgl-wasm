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

    console_log::init_with_level(log::Level::Debug).ok();
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
