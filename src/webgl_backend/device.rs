//! Device and context management

use glow::HasContext;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

/// Cached FBO with its depth renderbuffer
pub struct CachedFbo {
    pub fbo: glow::Framebuffer,
    pub depth_renderbuffer: glow::Renderbuffer,
    pub width: u32,
    pub height: u32,
}

/// Internal GL context wrapper
pub struct GlContext {
    pub gl: glow::Context,
    pub width: u32,
    pub height: u32,
    /// Cache of FBOs keyed by texture handle (for render-to-texture)
    pub fbo_cache: HashMap<glow::Texture, CachedFbo>,
    /// Reference to the canvas for getting current size
    pub canvas: HtmlCanvasElement,
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
        use super::types::buffer_usage;

        // Determine the correct GL target based on buffer usage
        let target = if buffer.usage & buffer_usage::INDEX != 0 {
            glow::ELEMENT_ARRAY_BUFFER
        } else if buffer.usage & buffer_usage::UNIFORM != 0 {
            glow::UNIFORM_BUFFER
        } else {
            glow::ARRAY_BUFFER
        };

        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.bind_buffer(target, Some(buffer.raw));
            ctx.gl.buffer_sub_data_u8_slice(
                target,
                offset as i32,
                data,
            );
            ctx.gl.bind_buffer(target, None);
        }
    }

    /// Write data to a texture
    ///
    /// texture: The destination texture
    /// mip_level: The mip level to write to
    /// origin_x, origin_y, origin_z: The origin within the texture to write to
    /// data: The pixel data to write
    /// bytes_per_row: Bytes per row in the source data (for alignment)
    /// rows_per_image: Rows per image (for 3D textures / 2D arrays)
    /// width, height, depth: Size of the region to write
    #[wasm_bindgen(js_name = writeTexture)]
    pub fn write_texture(
        &self,
        texture: &super::texture::WTexture,
        mip_level: u32,
        origin_x: u32,
        origin_y: u32,
        origin_z: u32,
        data: &[u8],
        bytes_per_row: u32,
        _rows_per_image: u32,
        width: u32,
        height: u32,
        depth: u32,
    ) {
        let Some(tex) = texture.raw else {
            log::warn!("Cannot write to surface texture");
            return;
        };

        let ctx = self.context.borrow();
        let format = texture.format;
        let gl_format = format.gl_format();
        let gl_type = format.gl_type();

        unsafe {
            // Set unpack alignment based on bytes_per_row
            // WebGL requires proper alignment for texture uploads
            let pixel_size = match format {
                super::texture::WTextureFormat::R8Unorm |
                super::texture::WTextureFormat::R8Snorm |
                super::texture::WTextureFormat::R8Uint |
                super::texture::WTextureFormat::R8Sint => 1,
                super::texture::WTextureFormat::Rg8Unorm |
                super::texture::WTextureFormat::Rg8Snorm |
                super::texture::WTextureFormat::Rg8Uint |
                super::texture::WTextureFormat::Rg8Sint => 2,
                _ => 4, // RGBA and depth formats
            };

            // Calculate expected row size and set row length if there's padding
            let expected_row_size = width * pixel_size;
            if bytes_per_row > expected_row_size {
                ctx.gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, (bytes_per_row / pixel_size) as i32);
            }

            // Determine if this is a 2D or 2D array texture
            let is_array = texture.depth_or_array_layers > 1;

            if is_array || depth > 1 {
                // 2D array texture or 3D texture
                ctx.gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(tex));
                ctx.gl.tex_sub_image_3d(
                    glow::TEXTURE_2D_ARRAY,
                    mip_level as i32,
                    origin_x as i32,
                    origin_y as i32,
                    origin_z as i32,
                    width as i32,
                    height as i32,
                    depth as i32,
                    gl_format,
                    gl_type,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
                ctx.gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
            } else {
                // Regular 2D texture
                ctx.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                ctx.gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    mip_level as i32,
                    origin_x as i32,
                    origin_y as i32,
                    width as i32,
                    height as i32,
                    gl_format,
                    gl_type,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
                ctx.gl.bind_texture(glow::TEXTURE_2D, None);
            }

            // Reset row length
            if bytes_per_row > expected_row_size {
                ctx.gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
            }
        }

        log::debug!(
            "Wrote {}x{}x{} pixels to texture at ({}, {}, {}), mip {}",
            width, height, depth, origin_x, origin_y, origin_z, mip_level
        );
    }
}

/// Create a device and queue from a canvas element
/// This is equivalent to adapter.requestDevice() + context.configure()
#[wasm_bindgen(js_name = createDevice)]
pub fn create_device(canvas: &HtmlCanvasElement) -> Result<WDevice, JsValue> {
    let width = canvas.width();
    let height = canvas.height();

    // Get WebGL2 context with explicit depth buffer
    let mut context_options = web_sys::WebGlContextAttributes::new();
    context_options.set_depth(true);
    context_options.set_antialias(false); // We handle MSAA ourselves if needed
    context_options.set_stencil(true);

    let webgl2_context = canvas
        .get_context_with_context_options("webgl2", &context_options)?
        .ok_or_else(|| JsValue::from_str("Failed to get WebGL2 context"))?
        .dyn_into::<web_sys::WebGl2RenderingContext>()?;

    // Create glow context from WebGL2
    let gl = glow::Context::from_webgl2_context(webgl2_context);

    log::info!("WebGL2 device created ({}x{})", width, height);

    let context = Rc::new(RefCell::new(GlContext {
        gl,
        width,
        height,
        fbo_cache: HashMap::new(),
        canvas: canvas.clone(),
    }));

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

    /// Get the current surface texture (default framebuffer)
    ///
    /// In WebGL, the "surface texture" is the default framebuffer (the canvas).
    /// This returns a WTexture with raw=None, which signals to the render pass
    /// that it should render to the default framebuffer.
    ///
    /// This is equivalent to GPUCanvasContext.getCurrentTexture() in WebGPU.
    #[wasm_bindgen(js_name = getSurfaceTexture)]
    pub fn get_surface_texture(&self) -> super::texture::WTexture {
        let mut ctx = self.context.borrow_mut();

        // Always read current canvas size to handle resize
        let canvas_width = ctx.canvas.width();
        let canvas_height = ctx.canvas.height();

        // Update stored dimensions if changed
        if ctx.width != canvas_width || ctx.height != canvas_height {
            log::info!("Canvas resized: {}x{} -> {}x{}", ctx.width, ctx.height, canvas_width, canvas_height);
            ctx.width = canvas_width;
            ctx.height = canvas_height;
        }

        super::texture::WTexture {
            raw: None, // None = default framebuffer
            width: canvas_width,
            height: canvas_height,
            depth_or_array_layers: 1,
            format: super::texture::WTextureFormat::Rgba8Unorm, // Canvas is typically RGBA8
            context: self.context.clone(),
            is_surface_texture: true,
        }
    }
}
