//! Device and Queue wrappers

use wasm_bindgen::prelude::*;
use std::sync::Arc;
use std::cell::RefCell;

/// Internal state shared between device operations
pub(crate) struct DeviceState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

// Thread-local storage for the current device state
// Since WASM is single-threaded, we use RefCell
thread_local! {
    static DEVICE_STATE: RefCell<Option<Arc<RefCell<DeviceState>>>> = const { RefCell::new(None) };
}

pub(crate) fn get_device_state() -> Arc<RefCell<DeviceState>> {
    DEVICE_STATE.with(|state| {
        state.borrow().as_ref().expect("Device not initialized").clone()
    })
}

fn set_device_state(state: Arc<RefCell<DeviceState>>) {
    DEVICE_STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

/// WebGPU Device wrapper
#[wasm_bindgen]
pub struct WDevice {
    state: Arc<RefCell<DeviceState>>,
}

impl WDevice {
    pub(crate) fn state(&self) -> Arc<RefCell<DeviceState>> {
        self.state.clone()
    }
}

/// WebGPU Queue wrapper
#[wasm_bindgen]
pub struct WQueue {
    state: Arc<RefCell<DeviceState>>,
}

impl WQueue {
    pub(crate) fn state(&self) -> Arc<RefCell<DeviceState>> {
        self.state.clone()
    }
}

use super::texture::{WTexture, WTextureFormat};

#[wasm_bindgen]
impl WDevice {
    /// Get the queue associated with this device
    #[wasm_bindgen(js_name = getQueue)]
    pub fn get_queue(&self) -> WQueue {
        WQueue {
            state: self.state.clone(),
        }
    }

    /// Get the current surface texture (default framebuffer)
    #[wasm_bindgen(js_name = getSurfaceTexture)]
    pub fn get_surface_texture(&self) -> WTexture {
        let state = self.state.borrow();
        WTexture {
            inner: None,
            is_surface: true,
            width: state.surface_config.width,
            height: state.surface_config.height,
            depth_or_array_layers: 1,
            format: WTextureFormat::Bgra8Unorm,
            mip_level_count: 1,
        }
    }
}

/// Create a device from a canvas element
/// If requested_format is provided and supported, it will be used; otherwise falls back to a supported format
/// If prefer_linear is true, prefers non-sRGB formats when falling back
#[wasm_bindgen(js_name = createDevice)]
pub async fn create_device(canvas: web_sys::HtmlCanvasElement, requested_format: Option<WTextureFormat>, prefer_linear: Option<bool>) -> Result<WDevice, JsValue> {
    let width = canvas.width();
    let height = canvas.height();

    log::info!("Creating wgpu device for canvas {}x{}", width, height);

    // Create wgpu instance with WebGL backend
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL,
        ..Default::default()
    });

    // Create surface from canvas
    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
        .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {:?}", e)))?;

    // Request adapter
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to get adapter: {:?}", e)))?;

    log::info!("Got adapter: {:?}", adapter.get_info());

    // Request device
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("wgpu-webgl-wasm device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
        })
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to get device: {:?}", e)))?;

    // Configure surface
    let surface_caps = surface.get_capabilities(&adapter);
    let prefer_linear = prefer_linear.unwrap_or(false);

    // Log available formats
    log::info!("Available surface formats: {:?}", surface_caps.formats);

    // Helper to select fallback format based on preference
    let select_fallback = |formats: &[wgpu::TextureFormat], prefer_linear: bool| -> wgpu::TextureFormat {
        if prefer_linear {
            // Prefer non-sRGB (linear) format
            formats
                .iter()
                .find(|f| !f.is_srgb())
                .copied()
                .unwrap_or(formats[0])
        } else {
            // Prefer sRGB format
            formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(formats[0])
        }
    };

    // If a format was requested, check if it's supported
    let surface_format = if let Some(req_format) = requested_format {
        let wgpu_format = req_format.to_wgpu();
        if surface_caps.formats.contains(&wgpu_format) {
            log::info!("Using requested format {:?}", wgpu_format);
            wgpu_format
        } else {
            let fallback = select_fallback(&surface_caps.formats, prefer_linear);
            log::warn!(
                "Requested format {:?} not supported. Available formats: {:?}. Falling back to {:?}",
                wgpu_format,
                surface_caps.formats,
                fallback
            );
            fallback
        }
    } else {
        // No format requested, use preference
        select_fallback(&surface_caps.formats, prefer_linear)
    };

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    log::info!("Device created successfully with format {:?}", surface_format);

    let state = Arc::new(RefCell::new(DeviceState {
        device,
        queue,
        surface,
        surface_config,
    }));

    set_device_state(state.clone());

    Ok(WDevice { state })
}

/// Update the viewport size (called when canvas resizes)
#[wasm_bindgen(js_name = setViewportSize)]
pub fn set_viewport_size(device: &WDevice, width: u32, height: u32) {
    let mut state = device.state.borrow_mut();

    // Only reconfigure if size actually changed
    if state.surface_config.width == width && state.surface_config.height == height {
        return;
    }

    state.surface_config.width = width;
    state.surface_config.height = height;

    // Reconfigure the surface with the new size
    state.surface.configure(&state.device, &state.surface_config);

    log::info!("Viewport resized to {}x{} and surface reconfigured", width, height);
}

use super::buffer::WBuffer;
use super::command::execute_pending_command_buffer;

#[wasm_bindgen]
impl WQueue {
    /// Submit command buffers - executes all recorded commands and presents the surface
    pub fn submit(&self) {
        log::debug!("Queue submit - executing pending command buffer");
        execute_pending_command_buffer();
    }

    /// Write data to a buffer
    #[wasm_bindgen(js_name = writeBuffer)]
    pub fn write_buffer(&self, buffer: &WBuffer, offset: u32, data: &[u8]) {
        let state = self.state.borrow();
        state.queue.write_buffer(buffer.inner(), offset as u64, data);
        log::debug!("Wrote {} bytes to buffer at offset {}", data.len(), offset);
    }

    /// Write data to a texture
    #[wasm_bindgen(js_name = writeTexture)]
    pub fn write_texture(
        &self,
        texture: &WTexture,
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
        let state = self.state.borrow();

        if let Some(ref tex) = texture.inner {
            state.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: origin_x,
                        y: origin_y,
                        z: origin_z,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: depth,
                },
            );
            log::debug!("Wrote texture data: {}x{}x{} at mip {}", width, height, depth, mip_level);
        } else {
            log::warn!("Cannot write to surface texture");
        }
    }
}
