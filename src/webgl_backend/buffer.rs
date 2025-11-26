//! Buffer management

use super::device::GlContextRef;
use super::types::buffer_usage;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// GPU Buffer - equivalent to GPUBuffer
#[wasm_bindgen]
pub struct WBuffer {
    context: GlContextRef,
    pub(crate) raw: glow::Buffer,
    pub(crate) size: u32,
    pub(crate) usage: u32,
}

impl Drop for WBuffer {
    fn drop(&mut self) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.delete_buffer(self.raw);
        }
        log::debug!("Buffer destroyed");
    }
}

impl WBuffer {
    pub fn context(&self) -> GlContextRef {
        self.context.clone()
    }
}

#[wasm_bindgen]
impl WBuffer {
    /// Get buffer size in bytes
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.size
    }
}

/// Create a buffer
#[wasm_bindgen(js_name = createBuffer)]
pub fn create_buffer(
    device: &super::WDevice,
    size: u32,
    usage: u32,
) -> Result<WBuffer, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        let buffer = ctx
            .gl
            .create_buffer()
            .map_err(|e| JsValue::from_str(&format!("Failed to create buffer: {}", e)))?;

        // Determine GL target based on usage
        let target = if usage & buffer_usage::INDEX != 0 {
            glow::ELEMENT_ARRAY_BUFFER
        } else if usage & buffer_usage::UNIFORM != 0 {
            glow::UNIFORM_BUFFER
        } else {
            glow::ARRAY_BUFFER
        };

        // Determine GL usage hint
        let gl_usage = if usage & (buffer_usage::MAP_READ | buffer_usage::MAP_WRITE) != 0 {
            glow::DYNAMIC_DRAW
        } else {
            glow::STATIC_DRAW
        };

        ctx.gl.bind_buffer(target, Some(buffer));
        ctx.gl.buffer_data_size(target, size as i32, gl_usage);
        ctx.gl.bind_buffer(target, None);

        log::debug!("Buffer created: {} bytes, usage: {:#x}", size, usage);

        Ok(WBuffer {
            context: context.clone(),
            raw: buffer,
            size,
            usage,
        })
    }
}

/// Create a buffer with initial data
#[wasm_bindgen(js_name = createBufferWithData)]
pub fn create_buffer_with_data(
    device: &super::WDevice,
    data: &[u8],
    usage: u32,
) -> Result<WBuffer, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        let buffer = ctx
            .gl
            .create_buffer()
            .map_err(|e| JsValue::from_str(&format!("Failed to create buffer: {}", e)))?;

        // Determine GL target based on usage
        let target = if usage & buffer_usage::INDEX != 0 {
            glow::ELEMENT_ARRAY_BUFFER
        } else if usage & buffer_usage::UNIFORM != 0 {
            glow::UNIFORM_BUFFER
        } else {
            glow::ARRAY_BUFFER
        };

        // Determine GL usage hint
        let gl_usage = if usage & (buffer_usage::MAP_READ | buffer_usage::MAP_WRITE) != 0 {
            glow::DYNAMIC_DRAW
        } else {
            glow::STATIC_DRAW
        };

        ctx.gl.bind_buffer(target, Some(buffer));
        ctx.gl.buffer_data_u8_slice(target, data, gl_usage);
        ctx.gl.bind_buffer(target, None);

        log::debug!("Buffer created with data: {} bytes, usage: {:#x}", data.len(), usage);

        Ok(WBuffer {
            context: context.clone(),
            raw: buffer,
            size: data.len() as u32,
            usage,
        })
    }
}
