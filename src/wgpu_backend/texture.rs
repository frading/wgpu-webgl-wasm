//! Texture and TextureView wrappers

use wasm_bindgen::prelude::*;
use std::sync::atomic::Ordering;
use super::device::{WDevice, WQueue};
use super::stats::{TEXTURE_COUNT, TEXTURE_VIEW_COUNT};

/// Texture format enum (matching WebGPU, values match .d.ts)
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum WTextureFormat {
    // 8-bit formats
    R8Unorm = 0,
    R8Snorm = 1,
    R8Uint = 2,
    R8Sint = 3,

    // RG 8-bit formats (values match .d.ts: 10-13)
    Rg8Unorm = 10,
    Rg8Snorm = 11,
    Rg8Uint = 12,
    Rg8Sint = 13,

    // RGBA 8-bit formats (values match .d.ts: 20-26)
    Rgba8Unorm = 20,
    Rgba8UnormSrgb = 21,
    Rgba8Snorm = 22,
    Rgba8Uint = 23,
    Rgba8Sint = 24,
    Bgra8Unorm = 25,
    Bgra8UnormSrgb = 26,

    // Depth formats (values match .d.ts: 50-53)
    Depth16Unorm = 50,
    Depth24Plus = 51,
    Depth24PlusStencil8 = 52,
    Depth32Float = 53,
}

impl WTextureFormat {
    pub(crate) fn to_wgpu(self) -> wgpu::TextureFormat {
        match self {
            Self::R8Unorm => wgpu::TextureFormat::R8Unorm,
            Self::R8Snorm => wgpu::TextureFormat::R8Snorm,
            Self::R8Uint => wgpu::TextureFormat::R8Uint,
            Self::R8Sint => wgpu::TextureFormat::R8Sint,
            Self::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            Self::Rg8Snorm => wgpu::TextureFormat::Rg8Snorm,
            Self::Rg8Uint => wgpu::TextureFormat::Rg8Uint,
            Self::Rg8Sint => wgpu::TextureFormat::Rg8Sint,
            Self::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            Self::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
            Self::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            Self::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
            Self::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            Self::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            Self::Depth16Unorm => wgpu::TextureFormat::Depth16Unorm,
            Self::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
            Self::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
            Self::Depth32Float => wgpu::TextureFormat::Depth32Float,
        }
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

impl WTextureDimension {
    pub(crate) fn to_wgpu(self) -> wgpu::TextureDimension {
        match self {
            Self::D1 => wgpu::TextureDimension::D1,
            Self::D2 => wgpu::TextureDimension::D2,
            Self::D3 => wgpu::TextureDimension::D3,
        }
    }
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

impl WTextureViewDimension {
    pub(crate) fn to_wgpu(self) -> wgpu::TextureViewDimension {
        match self {
            Self::D1 => wgpu::TextureViewDimension::D1,
            Self::D2 => wgpu::TextureViewDimension::D2,
            Self::D2Array => wgpu::TextureViewDimension::D2Array,
            Self::Cube => wgpu::TextureViewDimension::Cube,
            Self::CubeArray => wgpu::TextureViewDimension::CubeArray,
            Self::D3 => wgpu::TextureViewDimension::D3,
        }
    }
}

/// WebGPU Texture wrapper
#[wasm_bindgen]
pub struct WTexture {
    pub(crate) inner: Option<wgpu::Texture>,
    pub(crate) is_surface: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth_or_array_layers: u32,
    pub(crate) format: WTextureFormat,
    pub(crate) mip_level_count: u32,
}

impl WTexture {
    pub(crate) fn inner(&self) -> Option<&wgpu::Texture> {
        self.inner.as_ref()
    }
}

impl Drop for WTexture {
    fn drop(&mut self) {
        TEXTURE_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

#[wasm_bindgen]
impl WTexture {
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen(getter, js_name = depthOrArrayLayers)]
    pub fn depth_or_array_layers(&self) -> u32 {
        self.depth_or_array_layers
    }

    #[wasm_bindgen(getter)]
    pub fn format(&self) -> WTextureFormat {
        self.format
    }

    #[wasm_bindgen(js_name = createView)]
    pub fn create_view(&self) -> WTextureView {
        TEXTURE_VIEW_COUNT.fetch_add(1, Ordering::Relaxed);
        if self.is_surface {
            // For surface textures, we need to get the current frame
            WTextureView {
                inner: None,
                is_surface: true,
                width: self.width,
                height: self.height,
                format: self.format,
                dimension: WTextureViewDimension::D2,
            }
        } else if let Some(ref texture) = self.inner {
            // Explicitly set the dimension to avoid wgpu's heuristics
            // (e.g., 12 layers would be assumed CubeArray because 12 % 6 == 0)
            let view_dimension = if self.depth_or_array_layers > 1 {
                wgpu::TextureViewDimension::D2Array
            } else {
                wgpu::TextureViewDimension::D2
            };

            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: None,
                dimension: Some(view_dimension),
                usage: None,
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
            WTextureView {
                inner: Some(view),
                is_surface: false,
                width: self.width,
                height: self.height,
                format: self.format,
                dimension: if self.depth_or_array_layers > 1 {
                    WTextureViewDimension::D2Array
                } else {
                    WTextureViewDimension::D2
                },
            }
        } else {
            panic!("Cannot create view from null texture");
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
        TEXTURE_VIEW_COUNT.fetch_add(1, Ordering::Relaxed);
        if self.is_surface {
            WTextureView {
                inner: None,
                is_surface: true,
                width: self.width,
                height: self.height,
                format,
                dimension,
            }
        } else if let Some(ref texture) = self.inner {
            // Only specify format if it differs from texture format
            // Otherwise wgpu requires it in view_formats array
            let view_format = if format == self.format {
                None
            } else {
                Some(format.to_wgpu())
            };

            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: view_format,
                dimension: Some(dimension.to_wgpu()),
                usage: None,
                aspect: wgpu::TextureAspect::All,
                base_mip_level,
                mip_level_count: if mip_level_count == 0 { None } else { Some(mip_level_count) },
                base_array_layer,
                array_layer_count: if array_layer_count == 0 { None } else { Some(array_layer_count) },
            });
            WTextureView {
                inner: Some(view),
                is_surface: false,
                width: self.width >> base_mip_level,
                height: self.height >> base_mip_level,
                format: if format == self.format { self.format } else { format },
                dimension,
            }
        } else {
            panic!("Cannot create view from null texture");
        }
    }
}

/// WebGPU TextureView wrapper
#[wasm_bindgen]
pub struct WTextureView {
    pub(crate) inner: Option<wgpu::TextureView>,
    pub(crate) is_surface: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: WTextureFormat,
    pub(crate) dimension: WTextureViewDimension,
}

impl WTextureView {
    pub(crate) fn inner(&self) -> Option<&wgpu::TextureView> {
        self.inner.as_ref()
    }
}

impl Drop for WTextureView {
    fn drop(&mut self) {
        TEXTURE_VIEW_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

#[wasm_bindgen]
impl WTextureView {
    #[wasm_bindgen(getter, js_name = isSurfaceTexture)]
    pub fn is_surface_texture(&self) -> bool {
        self.is_surface
    }
}

/// Create a texture
#[wasm_bindgen(js_name = createTexture)]
pub fn create_texture(
    device: &WDevice,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    format: WTextureFormat,
    dimension: WTextureDimension,
    mip_level_count: u32,
    sample_count: u32,
    usage: u32,
) -> Result<WTexture, JsValue> {
    let state = device.state();
    let state = state.borrow();

    // Check for WebGL2 limitations with depth texture arrays
    let is_depth_format = matches!(
        format,
        WTextureFormat::Depth16Unorm
            | WTextureFormat::Depth24Plus
            | WTextureFormat::Depth24PlusStencil8
            | WTextureFormat::Depth32Float
    );

    if is_depth_format && depth_or_array_layers > 1 {
        log::warn!(
            "Creating depth texture array ({}x{}x{}, {:?}) - this may not be supported on WebGL2",
            width, height, depth_or_array_layers, format
        );
    }

    let texture = state.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        },
        mip_level_count: mip_level_count.max(1),
        sample_count: sample_count.max(1),
        dimension: dimension.to_wgpu(),
        format: format.to_wgpu(),
        usage: wgpu::TextureUsages::from_bits_truncate(usage),
        view_formats: &[],
    });

    log::debug!(
        "Created texture: {}x{}x{}, format={:?}, mips={}, samples={}",
        width, height, depth_or_array_layers, format, mip_level_count, sample_count
    );

    TEXTURE_COUNT.fetch_add(1, Ordering::Relaxed);

    Ok(WTexture {
        inner: Some(texture),
        is_surface: false,
        width,
        height,
        depth_or_array_layers,
        format,
        mip_level_count: mip_level_count.max(1),
    })
}

/// Get the current surface texture (for rendering to canvas)
/// Note: This is now a method on WDevice in the .d.ts
#[wasm_bindgen(js_name = getSurfaceTexture)]
pub fn get_surface_texture(device: &WDevice) -> WTexture {
    let state = device.state();
    let state = state.borrow();

    TEXTURE_COUNT.fetch_add(1, Ordering::Relaxed);

    WTexture {
        inner: None,
        is_surface: true,
        width: state.surface_config.width,
        height: state.surface_config.height,
        depth_or_array_layers: 1,
        format: WTextureFormat::Bgra8Unorm, // Will be overridden by actual surface format
        mip_level_count: 1,
    }
}

/// Write data to a texture
#[wasm_bindgen(js_name = writeTexture)]
pub fn write_texture(
    queue: &WQueue,
    texture: &WTexture,
    data: &[u8],
    bytes_per_row: u32,
    rows_per_image: u32,
    width: u32,
    height: u32,
    depth: u32,
) {
    let state = queue.state();
    let state = state.borrow();

    if let Some(ref tex) = texture.inner {
        state.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
        );

        log::debug!("Wrote texture data: {}x{}x{}", width, height, depth);
    } else {
        log::warn!("Cannot write to surface texture");
    }
}
