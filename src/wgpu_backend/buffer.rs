//! Buffer wrapper

use wasm_bindgen::prelude::*;
use super::device::{WDevice, WQueue};
use super::stats::BUFFER_COUNT;
use std::sync::atomic::Ordering;

/// Buffer usage flags (matching WebGPU)
pub mod buffer_usage {
    pub const MAP_READ: u32 = 1;
    pub const MAP_WRITE: u32 = 2;
    pub const COPY_SRC: u32 = 4;
    pub const COPY_DST: u32 = 8;
    pub const INDEX: u32 = 16;
    pub const VERTEX: u32 = 32;
    pub const UNIFORM: u32 = 64;
    pub const STORAGE: u32 = 128;
    pub const INDIRECT: u32 = 256;
    pub const QUERY_RESOLVE: u32 = 512;
}

/// WebGPU Buffer wrapper
#[wasm_bindgen]
pub struct WBuffer {
    pub(crate) inner: wgpu::Buffer,
    pub(crate) size: u64,
    pub(crate) usage: u32,
}

impl WBuffer {
    pub(crate) fn inner(&self) -> &wgpu::Buffer {
        &self.inner
    }

    pub(crate) fn new(inner: wgpu::Buffer, size: u64, usage: u32) -> Self {
        BUFFER_COUNT.fetch_add(1, Ordering::Relaxed);
        Self { inner, size, usage }
    }
}

impl Drop for WBuffer {
    fn drop(&mut self) {
        BUFFER_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

#[wasm_bindgen]
impl WBuffer {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.size as u32
    }
}

/// Create a buffer
#[wasm_bindgen(js_name = createBuffer)]
pub fn create_buffer(device: &WDevice, size: u64, usage: u32) -> WBuffer {
    let state = device.state();
    let state = state.borrow();

    let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::from_bits_truncate(usage),
        mapped_at_creation: false,
    });

    log::debug!("Created buffer: size={}, usage={:#x}", size, usage);

    WBuffer::new(buffer, size, usage)
}

/// Create a buffer with initial data
#[wasm_bindgen(js_name = createBufferWithData)]
pub fn create_buffer_with_data(device: &WDevice, data: &[u8], usage: u32) -> WBuffer {
    let state = device.state();
    let state = state.borrow();

    let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: data.len() as u64,
        usage: wgpu::BufferUsages::from_bits_truncate(usage) | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });

    // Write data to mapped buffer
    buffer.slice(..).get_mapped_range_mut().copy_from_slice(data);
    buffer.unmap();

    log::debug!("Created buffer with data: size={}, usage={:#x}", data.len(), usage);

    WBuffer::new(buffer, data.len() as u64, usage)
}

/// Write data to a buffer
#[wasm_bindgen(js_name = writeBuffer)]
pub fn write_buffer(queue: &WQueue, buffer: &WBuffer, offset: u64, data: &[u8]) {
    let state = queue.state();
    let state = state.borrow();

    state.queue.write_buffer(&buffer.inner, offset, data);

    log::debug!("Wrote {} bytes to buffer at offset {}", data.len(), offset);
}
