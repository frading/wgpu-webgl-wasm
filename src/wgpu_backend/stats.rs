//! Object counting and memory statistics for leak detection

use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

// Atomic counters for each object type
// Using i64 to detect underflows (negative = bug in counting)
pub static DEVICE_COUNT: AtomicI64 = AtomicI64::new(0);
pub static BUFFER_COUNT: AtomicI64 = AtomicI64::new(0);
pub static TEXTURE_COUNT: AtomicI64 = AtomicI64::new(0);
pub static TEXTURE_VIEW_COUNT: AtomicI64 = AtomicI64::new(0);
pub static SAMPLER_COUNT: AtomicI64 = AtomicI64::new(0);
pub static SHADER_MODULE_COUNT: AtomicI64 = AtomicI64::new(0);
pub static BIND_GROUP_LAYOUT_COUNT: AtomicI64 = AtomicI64::new(0);
pub static BIND_GROUP_COUNT: AtomicI64 = AtomicI64::new(0);
pub static PIPELINE_LAYOUT_COUNT: AtomicI64 = AtomicI64::new(0);
pub static RENDER_PIPELINE_COUNT: AtomicI64 = AtomicI64::new(0);
pub static COMMAND_ENCODER_COUNT: AtomicI64 = AtomicI64::new(0);
pub static RENDER_PIPELINE_DESCRIPTOR_COUNT: AtomicI64 = AtomicI64::new(0);
pub static RENDER_PASS_ENCODER_COUNT: AtomicI64 = AtomicI64::new(0);
pub static COMMAND_BUFFER_COUNT: AtomicI64 = AtomicI64::new(0);

// Memory tracking for strings and allocations
pub static STRING_BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

/// Returns object counts for all tracked types
#[wasm_bindgen(js_name = getObjectStats)]
pub fn get_object_stats() -> JsValue {
    let stats = js_sys::Object::new();

    let _ = js_sys::Reflect::set(&stats, &"devices".into(), &DEVICE_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"buffers".into(), &BUFFER_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"textures".into(), &TEXTURE_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"textureViews".into(), &TEXTURE_VIEW_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"samplers".into(), &SAMPLER_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"shaderModules".into(), &SHADER_MODULE_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"bindGroupLayouts".into(), &BIND_GROUP_LAYOUT_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"bindGroups".into(), &BIND_GROUP_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"pipelineLayouts".into(), &PIPELINE_LAYOUT_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"renderPipelines".into(), &RENDER_PIPELINE_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"commandEncoders".into(), &COMMAND_ENCODER_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"renderPipelineDescriptors".into(), &RENDER_PIPELINE_DESCRIPTOR_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"renderPassEncoders".into(), &RENDER_PASS_ENCODER_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"commandBuffers".into(), &COMMAND_BUFFER_COUNT.load(Ordering::Relaxed).into());
    let _ = js_sys::Reflect::set(&stats, &"stringBytesAllocated".into(), &(STRING_BYTES_ALLOCATED.load(Ordering::Relaxed) as u32).into());

    // Calculate total
    let total = DEVICE_COUNT.load(Ordering::Relaxed)
        + BUFFER_COUNT.load(Ordering::Relaxed)
        + TEXTURE_COUNT.load(Ordering::Relaxed)
        + TEXTURE_VIEW_COUNT.load(Ordering::Relaxed)
        + SAMPLER_COUNT.load(Ordering::Relaxed)
        + SHADER_MODULE_COUNT.load(Ordering::Relaxed)
        + BIND_GROUP_LAYOUT_COUNT.load(Ordering::Relaxed)
        + BIND_GROUP_COUNT.load(Ordering::Relaxed)
        + PIPELINE_LAYOUT_COUNT.load(Ordering::Relaxed)
        + RENDER_PIPELINE_COUNT.load(Ordering::Relaxed)
        + COMMAND_ENCODER_COUNT.load(Ordering::Relaxed)
        + RENDER_PIPELINE_DESCRIPTOR_COUNT.load(Ordering::Relaxed)
        + RENDER_PASS_ENCODER_COUNT.load(Ordering::Relaxed)
        + COMMAND_BUFFER_COUNT.load(Ordering::Relaxed);

    let _ = js_sys::Reflect::set(&stats, &"total".into(), &total.into());

    stats.into()
}

/// Resets all object counters to zero (for testing)
#[wasm_bindgen(js_name = resetObjectStats)]
pub fn reset_object_stats() {
    DEVICE_COUNT.store(0, Ordering::Relaxed);
    BUFFER_COUNT.store(0, Ordering::Relaxed);
    TEXTURE_COUNT.store(0, Ordering::Relaxed);
    TEXTURE_VIEW_COUNT.store(0, Ordering::Relaxed);
    SAMPLER_COUNT.store(0, Ordering::Relaxed);
    SHADER_MODULE_COUNT.store(0, Ordering::Relaxed);
    BIND_GROUP_LAYOUT_COUNT.store(0, Ordering::Relaxed);
    BIND_GROUP_COUNT.store(0, Ordering::Relaxed);
    PIPELINE_LAYOUT_COUNT.store(0, Ordering::Relaxed);
    RENDER_PIPELINE_COUNT.store(0, Ordering::Relaxed);
    COMMAND_ENCODER_COUNT.store(0, Ordering::Relaxed);
    RENDER_PIPELINE_DESCRIPTOR_COUNT.store(0, Ordering::Relaxed);
    RENDER_PASS_ENCODER_COUNT.store(0, Ordering::Relaxed);
    COMMAND_BUFFER_COUNT.store(0, Ordering::Relaxed);
    STRING_BYTES_ALLOCATED.store(0, Ordering::Relaxed);
}

/// Helper to track string allocation
pub fn track_string_alloc(s: &str) {
    STRING_BYTES_ALLOCATED.fetch_add(s.len(), Ordering::Relaxed);
}

/// Helper to track string deallocation
pub fn track_string_dealloc(len: usize) {
    STRING_BYTES_ALLOCATED.fetch_sub(len, Ordering::Relaxed);
}
