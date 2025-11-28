//! Texture and texture view management

use super::device::GlContextRef;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Texture format enum (subset of WebGPU formats supported by WebGL2)
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WTextureFormat {
    // 8-bit formats
    R8Unorm = 0,
    R8Snorm = 1,
    R8Uint = 2,
    R8Sint = 3,

    // 16-bit formats
    Rg8Unorm = 10,
    Rg8Snorm = 11,
    Rg8Uint = 12,
    Rg8Sint = 13,

    // 32-bit formats
    Rgba8Unorm = 20,
    Rgba8UnormSrgb = 21,
    Rgba8Snorm = 22,
    Rgba8Uint = 23,
    Rgba8Sint = 24,
    Bgra8Unorm = 25,
    Bgra8UnormSrgb = 26,

    // Depth/stencil formats
    Depth16Unorm = 50,
    Depth24Plus = 51,
    Depth24PlusStencil8 = 52,
    Depth32Float = 53,
}

impl Default for WTextureFormat {
    fn default() -> Self {
        WTextureFormat::Rgba8Unorm
    }
}

/// Texture dimension
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WTextureDimension {
    D1 = 0,
    D2 = 1,
    D3 = 2,
}

/// Texture view dimension
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WTextureViewDimension {
    D1 = 0,
    D2 = 1,
    D2Array = 2,
    Cube = 3,
    CubeArray = 4,
    D3 = 5,
}

/// WebGL2 Texture - equivalent to GPUTexture
///
/// When `raw` is None, this represents the default framebuffer (canvas surface).
#[wasm_bindgen]
pub struct WTexture {
    /// The GL texture handle. None means default framebuffer.
    pub(crate) raw: Option<glow::Texture>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth_or_array_layers: u32,
    pub(crate) format: WTextureFormat,
    pub(crate) context: GlContextRef,
    /// True if this represents the default framebuffer (surface texture)
    pub(crate) is_surface_texture: bool,
}

#[wasm_bindgen]
impl WTexture {
    /// Get texture width
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get texture height
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get depth or array layers
    #[wasm_bindgen(getter, js_name = depthOrArrayLayers)]
    pub fn depth_or_array_layers(&self) -> u32 {
        self.depth_or_array_layers
    }

    /// Create a texture view from this texture
    #[wasm_bindgen(js_name = createView)]
    pub fn create_view(&self) -> WTextureView {
        // Determine the correct view dimension based on the texture's array layers
        let dimension = if self.depth_or_array_layers > 1 {
            WTextureViewDimension::D2Array
        } else {
            WTextureViewDimension::D2
        };

        WTextureView {
            texture_raw: self.raw,
            format: self.format,
            dimension,
            base_mip_level: 0,
            mip_level_count: 1,
            base_array_layer: 0,
            array_layer_count: self.depth_or_array_layers,
            context: self.context.clone(),
            is_surface_texture: self.is_surface_texture,
            width: self.width,
            height: self.height,
        }
    }

    /// Create a texture view with descriptor parameters
    #[wasm_bindgen(js_name = createViewWithDescriptor)]
    pub fn create_view_with_descriptor(
        &self,
        format: WTextureFormat,
        dimension: WTextureViewDimension,
        base_mip_level: u32,
        mip_level_count: u32,
        base_array_layer: u32,
        array_layer_count: u32,
    ) -> WTextureView {
        WTextureView {
            texture_raw: self.raw,
            format,
            dimension,
            base_mip_level,
            mip_level_count,
            base_array_layer,
            array_layer_count,
            context: self.context.clone(),
            is_surface_texture: self.is_surface_texture,
            width: self.width,
            height: self.height,
        }
    }
}

/// WebGL2 TextureView - equivalent to GPUTextureView
///
/// When `texture_raw` is None and `is_surface_texture` is true,
/// this represents a view of the default framebuffer.
#[wasm_bindgen]
pub struct WTextureView {
    pub(crate) texture_raw: Option<glow::Texture>,
    pub(crate) format: WTextureFormat,
    pub(crate) dimension: WTextureViewDimension,
    pub(crate) base_mip_level: u32,
    pub(crate) mip_level_count: u32,
    pub(crate) base_array_layer: u32,
    pub(crate) array_layer_count: u32,
    pub(crate) context: GlContextRef,
    pub(crate) is_surface_texture: bool,
    /// Width of the texture (needed for viewport)
    pub(crate) width: u32,
    /// Height of the texture (needed for viewport)
    pub(crate) height: u32,
}

impl WTextureView {
    /// Check if this view targets the default framebuffer (surface)
    pub fn is_surface(&self) -> bool {
        self.is_surface_texture
    }

    /// Get the raw GL texture handle (None for surface texture)
    pub fn raw(&self) -> Option<glow::Texture> {
        self.texture_raw
    }
}

#[wasm_bindgen]
impl WTextureView {
    /// Check if this is a surface texture view (renders to canvas)
    #[wasm_bindgen(getter, js_name = isSurfaceTexture)]
    pub fn is_surface_texture_js(&self) -> bool {
        self.is_surface_texture
    }
}

impl Drop for WTexture {
    fn drop(&mut self) {
        if let Some(raw) = self.raw {
            let ctx = self.context.borrow();
            unsafe {
                ctx.gl.delete_texture(raw);
            }
            log::debug!("Texture destroyed");
        }
    }
}

/// Texture usage flags
pub mod texture_usage {
    pub const COPY_SRC: u32 = 0x01;
    pub const COPY_DST: u32 = 0x02;
    pub const TEXTURE_BINDING: u32 = 0x04;
    pub const STORAGE_BINDING: u32 = 0x08;
    pub const RENDER_ATTACHMENT: u32 = 0x10;
}

impl WTextureFormat {
    /// Get the GL internal format for this texture format
    pub fn gl_internal_format(self) -> u32 {
        match self {
            // 8-bit formats
            WTextureFormat::R8Unorm => glow::R8,
            WTextureFormat::R8Snorm => glow::R8_SNORM,
            WTextureFormat::R8Uint => glow::R8UI,
            WTextureFormat::R8Sint => glow::R8I,
            // 16-bit formats
            WTextureFormat::Rg8Unorm => glow::RG8,
            WTextureFormat::Rg8Snorm => glow::RG8_SNORM,
            WTextureFormat::Rg8Uint => glow::RG8UI,
            WTextureFormat::Rg8Sint => glow::RG8I,
            // 32-bit formats
            WTextureFormat::Rgba8Unorm => glow::RGBA8,
            WTextureFormat::Rgba8UnormSrgb => glow::SRGB8_ALPHA8,
            WTextureFormat::Rgba8Snorm => glow::RGBA8_SNORM,
            WTextureFormat::Rgba8Uint => glow::RGBA8UI,
            WTextureFormat::Rgba8Sint => glow::RGBA8I,
            WTextureFormat::Bgra8Unorm => glow::RGBA8, // WebGL2 doesn't have BGRA internal format
            WTextureFormat::Bgra8UnormSrgb => glow::SRGB8_ALPHA8,
            // Depth/stencil formats
            WTextureFormat::Depth16Unorm => glow::DEPTH_COMPONENT16,
            WTextureFormat::Depth24Plus => glow::DEPTH_COMPONENT24,
            WTextureFormat::Depth24PlusStencil8 => glow::DEPTH24_STENCIL8,
            WTextureFormat::Depth32Float => glow::DEPTH_COMPONENT32F,
        }
    }

    /// Get the GL format for this texture format (for glTexImage2D)
    pub fn gl_format(self) -> u32 {
        match self {
            // Red channel
            WTextureFormat::R8Unorm | WTextureFormat::R8Snorm => glow::RED,
            WTextureFormat::R8Uint | WTextureFormat::R8Sint => glow::RED_INTEGER,
            // RG channels
            WTextureFormat::Rg8Unorm | WTextureFormat::Rg8Snorm => glow::RG,
            WTextureFormat::Rg8Uint | WTextureFormat::Rg8Sint => glow::RG_INTEGER,
            // RGBA channels
            WTextureFormat::Rgba8Unorm | WTextureFormat::Rgba8UnormSrgb |
            WTextureFormat::Rgba8Snorm | WTextureFormat::Bgra8Unorm |
            WTextureFormat::Bgra8UnormSrgb => glow::RGBA,
            WTextureFormat::Rgba8Uint | WTextureFormat::Rgba8Sint => glow::RGBA_INTEGER,
            // Depth/stencil
            WTextureFormat::Depth16Unorm | WTextureFormat::Depth24Plus |
            WTextureFormat::Depth32Float => glow::DEPTH_COMPONENT,
            WTextureFormat::Depth24PlusStencil8 => glow::DEPTH_STENCIL,
        }
    }

    /// Get the GL type for this texture format
    pub fn gl_type(self) -> u32 {
        match self {
            WTextureFormat::R8Unorm | WTextureFormat::Rg8Unorm |
            WTextureFormat::Rgba8Unorm | WTextureFormat::Rgba8UnormSrgb |
            WTextureFormat::Bgra8Unorm | WTextureFormat::Bgra8UnormSrgb |
            WTextureFormat::R8Uint | WTextureFormat::Rg8Uint | WTextureFormat::Rgba8Uint => glow::UNSIGNED_BYTE,

            WTextureFormat::R8Snorm | WTextureFormat::Rg8Snorm |
            WTextureFormat::Rgba8Snorm | WTextureFormat::R8Sint |
            WTextureFormat::Rg8Sint | WTextureFormat::Rgba8Sint => glow::BYTE,

            WTextureFormat::Depth16Unorm => glow::UNSIGNED_SHORT,
            WTextureFormat::Depth24Plus => glow::UNSIGNED_INT,
            WTextureFormat::Depth24PlusStencil8 => glow::UNSIGNED_INT_24_8,
            WTextureFormat::Depth32Float => glow::FLOAT,
        }
    }

    /// Check if this is a depth or stencil format
    pub fn is_depth_stencil(self) -> bool {
        matches!(self,
            WTextureFormat::Depth16Unorm |
            WTextureFormat::Depth24Plus |
            WTextureFormat::Depth24PlusStencil8 |
            WTextureFormat::Depth32Float
        )
    }
}

/// Create a texture
#[wasm_bindgen(js_name = createTexture)]
pub fn create_texture(
    device: &super::WDevice,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    format: WTextureFormat,
    dimension: WTextureDimension,
    mip_level_count: u32,
    sample_count: u32,
    _usage: u32, // Usage flags (for compatibility, not strictly enforced in WebGL)
) -> Result<WTexture, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        let texture = ctx
            .gl
            .create_texture()
            .map_err(|e| JsValue::from_str(&format!("Failed to create texture: {}", e)))?;

        // Determine GL texture target based on dimension and array layers
        let target = match dimension {
            WTextureDimension::D1 => glow::TEXTURE_2D, // WebGL2 doesn't have 1D textures
            WTextureDimension::D2 => {
                if depth_or_array_layers > 1 {
                    glow::TEXTURE_2D_ARRAY
                } else {
                    glow::TEXTURE_2D
                }
            }
            WTextureDimension::D3 => glow::TEXTURE_3D,
        };

        ctx.gl.bind_texture(target, Some(texture));

        let internal_format = format.gl_internal_format();
        let gl_format = format.gl_format();
        let gl_type = format.gl_type();

        match target {
            glow::TEXTURE_2D => {
                // Allocate storage for 2D texture with mipmaps
                ctx.gl.tex_storage_2d(
                    target,
                    mip_level_count as i32,
                    internal_format,
                    width as i32,
                    height as i32,
                );
                let _ = sample_count; // For future multisampling support
            }
            glow::TEXTURE_2D_ARRAY => {
                ctx.gl.tex_storage_3d(
                    target,
                    mip_level_count as i32,
                    internal_format,
                    width as i32,
                    height as i32,
                    depth_or_array_layers as i32,
                );
            }
            glow::TEXTURE_3D => {
                ctx.gl.tex_storage_3d(
                    target,
                    mip_level_count as i32,
                    internal_format,
                    width as i32,
                    height as i32,
                    depth_or_array_layers as i32,
                );
            }
            _ => {}
        }

        // Set default sampling parameters
        ctx.gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        ctx.gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        ctx.gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        ctx.gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

        ctx.gl.bind_texture(target, None);

        log::info!(
            "Texture created: {}x{}x{}, format={:?} (internal={}, glFormat={}, glType={}), dimension={:?}, mips={}, usage={}",
            width, height, depth_or_array_layers, format,
            internal_format, gl_format, gl_type,
            dimension, mip_level_count, _usage
        );

        Ok(WTexture {
            raw: Some(texture),
            width,
            height,
            depth_or_array_layers,
            format,
            context: context.clone(),
            is_surface_texture: false,
        })
    }
}
