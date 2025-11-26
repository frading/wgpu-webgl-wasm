//! Device and context management

use glow::HasContext;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

/// Internal GL context wrapper
pub struct GlContext {
    pub gl: glow::Context,
    pub width: u32,
    pub height: u32,
}

/// Shared reference to GL context
pub type GlContextRef = Rc<RefCell<GlContext>>;

/// WebGL2 Device - equivalent to GPUDevice
#[wasm_bindgen]
pub struct WDevice {
    context: GlContextRef,
}

impl WDevice {
    pub fn context(&self) -> GlContextRef {
        self.context.clone()
    }
}

/// WebGL2 Queue - equivalent to GPUQueue
#[wasm_bindgen]
pub struct WQueue {
    context: GlContextRef,
}

impl WQueue {
    pub fn context(&self) -> GlContextRef {
        self.context.clone()
    }
}

#[wasm_bindgen]
impl WQueue {
    /// Submit command buffers for execution
    /// In WebGL, commands are executed immediately, so this is mostly a no-op
    /// but we flush to ensure commands are sent to the GPU
    pub fn submit(&self) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.flush();
        }
    }

    /// Write data to a buffer
    #[wasm_bindgen(js_name = writeBuffer)]
    pub fn write_buffer(&self, buffer: &super::WBuffer, offset: u32, data: &[u8]) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer.raw));
            ctx.gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                offset as i32,
                data,
            );
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }
}

/// Create a device and queue from a canvas element
/// This is equivalent to adapter.requestDevice() + context.configure()
#[wasm_bindgen(js_name = createDevice)]
pub fn create_device(canvas: &HtmlCanvasElement) -> Result<WDevice, JsValue> {
    let width = canvas.width();
    let height = canvas.height();

    // Get WebGL2 context
    let webgl2_context = canvas
        .get_context("webgl2")?
        .ok_or_else(|| JsValue::from_str("Failed to get WebGL2 context"))?
        .dyn_into::<web_sys::WebGl2RenderingContext>()?;

    // Create glow context from WebGL2
    let gl = glow::Context::from_webgl2_context(webgl2_context);

    log::info!("WebGL2 device created ({}x{})", width, height);

    let context = Rc::new(RefCell::new(GlContext { gl, width, height }));

    Ok(WDevice { context })
}

#[wasm_bindgen]
impl WDevice {
    /// Get the queue associated with this device
    #[wasm_bindgen(js_name = getQueue)]
    pub fn get_queue(&self) -> WQueue {
        WQueue {
            context: self.context.clone(),
        }
    }

    /// Update the viewport size (call when canvas resizes)
    #[wasm_bindgen(js_name = setViewportSize)]
    pub fn set_viewport_size(&self, width: u32, height: u32) {
        let mut ctx = self.context.borrow_mut();
        ctx.width = width;
        ctx.height = height;
        log::debug!("Viewport size updated to {}x{}", width, height);
    }
}
