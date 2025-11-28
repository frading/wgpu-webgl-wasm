//! WGPU WebGL2 WASM Bridge
//!
//! This module provides a WebGPU-like API that translates to WebGL2 calls.
//! It mirrors the WebGPU API so that JS can use the same calling pattern
//! regardless of whether native WebGPU or this WebGL2 fallback is used.

use wasm_bindgen::prelude::*;

mod webgl_backend;

pub use webgl_backend::*;

// Initialize console logging for debugging
#[wasm_bindgen(start)]
pub fn init() {
    // console_log::init_with_level(log::Level::Debug).ok();
    // console_log::init_with_level(log::Level::Info).ok();
    console_log::init_with_level(log::Level::Error).ok();
    log::info!("wgpu-webgl-wasm initialized");
}

/// A simple test function to verify the WASM module loads correctly
#[wasm_bindgen]
pub fn test_wasm() -> String {
    "wgpu-webgl-wasm is working!".to_string()
}
