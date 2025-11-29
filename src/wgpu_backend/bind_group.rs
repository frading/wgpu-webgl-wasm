//! Bind group and bind group layout wrappers

use wasm_bindgen::prelude::*;
use super::device::WDevice;
use super::buffer::WBuffer;
use super::texture::WTextureView;
use super::sampler::WSampler;

use std::sync::atomic::{AtomicU32, Ordering};

static BUILDER_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Bind group builder - accumulates entries and then creates the bind group
#[wasm_bindgen]
pub struct WBindGroupBuilder {
    id: u32,
    entries: Vec<BindGroupBuilderEntry>,
}

struct BindGroupBuilderEntry {
    binding: u32,
    entry_type: BindGroupEntryType,
}

enum BindGroupEntryType {
    Buffer {
        buffer: wgpu::Buffer,
        offset: u64,
        size: u64,
    },
    Sampler(wgpu::Sampler),
    TextureView(wgpu::TextureView),
}

#[wasm_bindgen]
impl WBindGroupBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WBindGroupBuilder {
        let id = BUILDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        log::info!("Created WBindGroupBuilder #{}", id);
        WBindGroupBuilder {
            id,
            entries: Vec::new(),
        }
    }

    /// Add a buffer entry
    #[wasm_bindgen(js_name = addBuffer)]
    pub fn add_buffer(&mut self, binding: u32, buffer: &WBuffer, offset: u64, size: u64) {
        log::info!("Builder #{}: addBuffer binding={}, offset={}, size={}", self.id, binding, offset, size);
        self.entries.push(BindGroupBuilderEntry {
            binding,
            entry_type: BindGroupEntryType::Buffer {
                buffer: buffer.inner().clone(),
                offset,
                size,
            },
        });
    }

    /// Add a sampler entry
    #[wasm_bindgen(js_name = addSampler)]
    pub fn add_sampler(&mut self, binding: u32, sampler: &WSampler) {
        log::info!("Builder #{}: addSampler binding={}", self.id, binding);
        self.entries.push(BindGroupBuilderEntry {
            binding,
            entry_type: BindGroupEntryType::Sampler(sampler.inner().clone()),
        });
    }

    /// Add a texture view entry
    #[wasm_bindgen(js_name = addTextureView)]
    pub fn add_texture_view(&mut self, binding: u32, texture_view: &WTextureView) -> Result<(), JsValue> {
        let view = texture_view
            .inner()
            .ok_or_else(|| JsValue::from_str("Cannot bind surface texture view"))?
            .clone();
        log::info!("Builder #{}: addTextureView binding={}", self.id, binding);
        self.entries.push(BindGroupBuilderEntry {
            binding,
            entry_type: BindGroupEntryType::TextureView(view),
        });
        Ok(())
    }

    /// Build the bind group (consumes the builder)
    #[wasm_bindgen]
    pub fn build(self, device: &WDevice, layout: &WBindGroupLayout) -> WBindGroup {
        let state = device.state();
        let state = state.borrow();

        // Log entry details for debugging
        log::info!("Builder #{}: Building bind group with {} entries:", self.id, self.entries.len());
        for entry in &self.entries {
            let type_name = match &entry.entry_type {
                BindGroupEntryType::Buffer { size, .. } => format!("Buffer(size={})", size),
                BindGroupEntryType::Sampler(_) => "Sampler".to_string(),
                BindGroupEntryType::TextureView(_) => "TextureView".to_string(),
            };
            log::info!("  binding={}, type={}", entry.binding, type_name);
        }

        // Convert to wgpu entries
        let wgpu_entries: Vec<wgpu::BindGroupEntry> = self.entries
            .iter()
            .map(|entry| {
                let resource = match &entry.entry_type {
                    BindGroupEntryType::Buffer { buffer, offset, size } => {
                        log::info!("  Creating wgpu entry: binding={}, resource=Buffer", entry.binding);
                        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer,
                            offset: *offset,
                            size: std::num::NonZeroU64::new(*size),
                        })
                    }
                    BindGroupEntryType::Sampler(sampler) => {
                        log::info!("  Creating wgpu entry: binding={}, resource=Sampler", entry.binding);
                        wgpu::BindingResource::Sampler(sampler)
                    }
                    BindGroupEntryType::TextureView(view) => {
                        log::info!("  Creating wgpu entry: binding={}, resource=TextureView", entry.binding);
                        wgpu::BindingResource::TextureView(view)
                    }
                };
                wgpu::BindGroupEntry {
                    binding: entry.binding,
                    resource,
                }
            })
            .collect();

        log::info!("Builder #{}: About to call device.create_bind_group with {} wgpu_entries", self.id, wgpu_entries.len());

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout.inner,
            entries: &wgpu_entries,
        });

        log::debug!("Created bind group with {} entries", self.entries.len());

        WBindGroup { inner: bind_group }
    }
}

/// Binding type
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WBindingType {
    UniformBuffer = 0,
    StorageBuffer = 1,
    StorageBufferReadWrite = 2,
    Sampler = 3,
    SampledTexture = 4,
    StorageTexture = 5,
}

/// Bind group layout entry
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WBindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: u32,
    pub binding_type: WBindingType,
    pub has_dynamic_offset: bool,
    pub min_binding_size: u64,
}

#[wasm_bindgen]
impl WBindGroupLayoutEntry {
    #[wasm_bindgen(constructor)]
    pub fn new(binding: u32, visibility: u32, binding_type: WBindingType) -> Self {
        Self {
            binding,
            visibility,
            binding_type,
            has_dynamic_offset: false,
            min_binding_size: 0,
        }
    }
}

/// Bind group layout
#[wasm_bindgen]
pub struct WBindGroupLayout {
    pub(crate) inner: wgpu::BindGroupLayout,
    pub(crate) entry_count: u32,
}

#[wasm_bindgen]
impl WBindGroupLayout {
    #[wasm_bindgen(getter, js_name = entryCount)]
    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }
}

/// Pipeline layout
#[wasm_bindgen]
pub struct WPipelineLayout {
    pub(crate) inner: wgpu::PipelineLayout,
    pub(crate) bind_group_layout_count: u32,
}

impl WPipelineLayout {
    pub(crate) fn inner(&self) -> &wgpu::PipelineLayout {
        &self.inner
    }
}

#[wasm_bindgen]
impl WPipelineLayout {
    #[wasm_bindgen(getter, js_name = bindGroupLayoutCount)]
    pub fn bind_group_layout_count(&self) -> u32 {
        self.bind_group_layout_count
    }
}

/// Bind group
#[wasm_bindgen]
pub struct WBindGroup {
    pub(crate) inner: wgpu::BindGroup,
}

impl WBindGroup {
    pub(crate) fn inner(&self) -> &wgpu::BindGroup {
        &self.inner
    }
}

/// Create a bind group layout from JS description
#[wasm_bindgen(js_name = createBindGroupLayout)]
pub fn create_bind_group_layout(
    device: &WDevice,
    entries_js: JsValue,
) -> Result<WBindGroupLayout, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let entries_array: js_sys::Array = entries_js
        .dyn_into()
        .map_err(|_| JsValue::from_str("entries must be an array"))?;

    let mut entries = Vec::new();

    for i in 0..entries_array.length() {
        let entry_obj = entries_array.get(i);

        let binding = js_sys::Reflect::get(&entry_obj, &"binding".into())
            .map_err(|_| JsValue::from_str("entry missing 'binding'"))?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("binding must be a number"))? as u32;

        let visibility = js_sys::Reflect::get(&entry_obj, &"visibility".into())
            .map_err(|_| JsValue::from_str("entry missing 'visibility'"))?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("visibility must be a number"))? as u32;

        // Determine binding type - check if property exists AND is truthy (not undefined/null)
        let buffer_val = js_sys::Reflect::get(&entry_obj, &"buffer".into()).ok();
        let sampler_val = js_sys::Reflect::get(&entry_obj, &"sampler".into()).ok();
        let texture_val = js_sys::Reflect::get(&entry_obj, &"texture".into()).ok();

        let has_buffer = buffer_val.map(|v| v.is_object()).unwrap_or(false);
        let has_sampler = sampler_val.map(|v| v.is_object()).unwrap_or(false);
        let has_texture = texture_val.map(|v| v.is_object()).unwrap_or(false);

        log::info!("createBindGroupLayout entry {}: binding={}, has_buffer={}, has_sampler={}, has_texture={}",
            i, binding, has_buffer, has_sampler, has_texture);

        let ty = if has_buffer {
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }
        } else if has_sampler {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
        } else if has_texture {
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            }
        } else {
            log::warn!("createBindGroupLayout entry {}: no recognized type, defaulting to Buffer", i);
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }
        };

        entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::from_bits_truncate(visibility),
            ty,
            count: None,
        });
    }

    let entry_count = entries.len() as u32;

    let layout = state
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });

    log::debug!("Created bind group layout with {} entries", entry_count);

    Ok(WBindGroupLayout { inner: layout, entry_count })
}

/// Create a pipeline layout with a single bind group layout
#[wasm_bindgen(js_name = createPipelineLayout)]
pub fn create_pipeline_layout(
    device: &WDevice,
    bind_group_layout: &WBindGroupLayout,
) -> WPipelineLayout {
    let state = device.state();
    let state = state.borrow();

    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout.inner],
            push_constant_ranges: &[],
        });

    log::info!(
        "Created pipeline layout with 1 bind group layout"
    );

    WPipelineLayout { inner: layout, bind_group_layout_count: 1 }
}

/// Create a pipeline layout with 2 bind group layouts
#[wasm_bindgen(js_name = createPipelineLayout2)]
pub fn create_pipeline_layout_2(
    device: &WDevice,
    layout0: &WBindGroupLayout,
    layout1: &WBindGroupLayout,
) -> WPipelineLayout {
    let state = device.state();
    let state = state.borrow();

    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout0.inner, &layout1.inner],
            push_constant_ranges: &[],
        });

    log::info!("Created pipeline layout with 2 bind group layouts");

    WPipelineLayout { inner: layout, bind_group_layout_count: 2 }
}

/// Create a pipeline layout with 3 bind group layouts
#[wasm_bindgen(js_name = createPipelineLayout3)]
pub fn create_pipeline_layout_3(
    device: &WDevice,
    layout0: &WBindGroupLayout,
    layout1: &WBindGroupLayout,
    layout2: &WBindGroupLayout,
) -> WPipelineLayout {
    let state = device.state();
    let state = state.borrow();

    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout0.inner, &layout1.inner, &layout2.inner],
            push_constant_ranges: &[],
        });

    log::info!("Created pipeline layout with 3 bind group layouts");

    WPipelineLayout { inner: layout, bind_group_layout_count: 3 }
}

/// Create a pipeline layout with 4 bind group layouts
#[wasm_bindgen(js_name = createPipelineLayout4)]
pub fn create_pipeline_layout_4(
    device: &WDevice,
    layout0: &WBindGroupLayout,
    layout1: &WBindGroupLayout,
    layout2: &WBindGroupLayout,
    layout3: &WBindGroupLayout,
) -> WPipelineLayout {
    let state = device.state();
    let state = state.borrow();

    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout0.inner, &layout1.inner, &layout2.inner, &layout3.inner],
            push_constant_ranges: &[],
        });

    log::info!("Created pipeline layout with 4 bind group layouts");

    WPipelineLayout { inner: layout, bind_group_layout_count: 4 }
}

/// Create a bind group with a single buffer
#[wasm_bindgen(js_name = createBindGroupWithBuffer)]
pub fn create_bind_group_with_buffer(
    device: &WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    buffer: &WBuffer,
    offset: u64,
    size: u64,
) -> WBindGroup {
    let state = device.state();
    let state = state.borrow();

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: buffer.inner(),
                offset,
                size: std::num::NonZeroU64::new(size),
            }),
        }],
    });

    log::debug!("Created bind group with buffer at binding {}", binding);

    WBindGroup { inner: bind_group }
}

/// Create a bind group with texture and sampler
#[wasm_bindgen(js_name = createBindGroupWithTextureViewSampler)]
pub fn create_bind_group_with_texture_view_sampler(
    device: &WDevice,
    layout: &WBindGroupLayout,
    texture_binding: u32,
    texture_view: &WTextureView,
    sampler_binding: u32,
    sampler: &WSampler,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let view = texture_view
        .inner()
        .ok_or_else(|| JsValue::from_str("Cannot bind surface texture view"))?;

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: texture_binding,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(sampler.inner()),
            },
        ],
    });

    log::debug!(
        "Created bind group with texture at {} and sampler at {}",
        texture_binding,
        sampler_binding
    );

    Ok(WBindGroup { inner: bind_group })
}

/// Create a bind group with buffer, texture, and sampler
#[wasm_bindgen(js_name = createBindGroupWithBufferTextureViewSampler)]
pub fn create_bind_group_with_buffer_texture_view_sampler(
    device: &WDevice,
    layout: &WBindGroupLayout,
    buffer_binding: u32,
    buffer: &WBuffer,
    buffer_offset: u64,
    buffer_size: u64,
    texture_binding: u32,
    texture_view: &WTextureView,
    sampler_binding: u32,
    sampler: &WSampler,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let view = texture_view
        .inner()
        .ok_or_else(|| JsValue::from_str("Cannot bind surface texture view"))?;

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: buffer_binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer.inner(),
                    offset: buffer_offset,
                    size: std::num::NonZeroU64::new(buffer_size),
                }),
            },
            wgpu::BindGroupEntry {
                binding: texture_binding,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(sampler.inner()),
            },
        ],
    });

    log::debug!(
        "Created bind group with buffer at {}, texture at {}, sampler at {}",
        buffer_binding,
        texture_binding,
        sampler_binding
    );

    Ok(WBindGroup { inner: bind_group })
}

/// Create a bind group with two buffer bindings
#[wasm_bindgen(js_name = createBindGroupWith2Buffers)]
pub fn create_bind_group_with_2_buffers(
    device: &WDevice,
    layout: &WBindGroupLayout,
    binding0: u32,
    buffer0: &WBuffer,
    offset0: u64,
    size0: u64,
    binding1: u32,
    buffer1: &WBuffer,
    offset1: u64,
    size1: u64,
) -> WBindGroup {
    let state = device.state();
    let state = state.borrow();

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: binding0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer0.inner(),
                    offset: offset0,
                    size: std::num::NonZeroU64::new(size0),
                }),
            },
            wgpu::BindGroupEntry {
                binding: binding1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer1.inner(),
                    offset: offset1,
                    size: std::num::NonZeroU64::new(size1),
                }),
            },
        ],
    });

    log::debug!("Created bind group with 2 buffers");
    WBindGroup { inner: bind_group }
}

/// Create a bind group with three buffer bindings
#[wasm_bindgen(js_name = createBindGroupWith3Buffers)]
pub fn create_bind_group_with_3_buffers(
    device: &WDevice,
    layout: &WBindGroupLayout,
    binding0: u32,
    buffer0: &WBuffer,
    offset0: u64,
    size0: u64,
    binding1: u32,
    buffer1: &WBuffer,
    offset1: u64,
    size1: u64,
    binding2: u32,
    buffer2: &WBuffer,
    offset2: u64,
    size2: u64,
) -> WBindGroup {
    let state = device.state();
    let state = state.borrow();

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: binding0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer0.inner(),
                    offset: offset0,
                    size: std::num::NonZeroU64::new(size0),
                }),
            },
            wgpu::BindGroupEntry {
                binding: binding1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer1.inner(),
                    offset: offset1,
                    size: std::num::NonZeroU64::new(size1),
                }),
            },
            wgpu::BindGroupEntry {
                binding: binding2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer2.inner(),
                    offset: offset2,
                    size: std::num::NonZeroU64::new(size2),
                }),
            },
        ],
    });

    log::debug!("Created bind group with 3 buffers");
    WBindGroup { inner: bind_group }
}

/// Create an empty bind group
#[wasm_bindgen(js_name = createEmptyBindGroup)]
pub fn create_empty_bind_group(device: &WDevice, layout: &WBindGroupLayout) -> WBindGroup {
    let state = device.state();
    let state = state.borrow();

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[],
    });

    log::debug!("Created empty bind group");
    WBindGroup { inner: bind_group }
}

/// Create a bind group with a texture view binding
#[wasm_bindgen(js_name = createBindGroupWithTextureView)]
pub fn create_bind_group_with_texture_view(
    device: &WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    texture_view: &WTextureView,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let view = texture_view
        .inner()
        .ok_or_else(|| JsValue::from_str("Cannot bind surface texture view"))?;

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        }],
    });

    log::debug!("Created bind group with texture view at binding {}", binding);
    Ok(WBindGroup { inner: bind_group })
}

/// Create a bind group with a sampler binding only
#[wasm_bindgen(js_name = createBindGroupWithSampler)]
pub fn create_bind_group_with_sampler(
    device: &WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    sampler: &WSampler,
) -> WBindGroup {
    let state = device.state();
    let state = state.borrow();

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler.inner()),
        }],
    });

    log::debug!("Created bind group with sampler at binding {}", binding);
    WBindGroup { inner: bind_group }
}

use super::texture::WTexture;

/// Create a bind group with a texture binding only
#[wasm_bindgen(js_name = createBindGroupWithTexture)]
pub fn create_bind_group_with_texture(
    device: &WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    texture: &WTexture,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let tex = texture
        .inner()
        .ok_or_else(|| JsValue::from_str("Cannot bind surface texture"))?;

    // Create a default view from the texture
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });

    log::debug!("Created bind group with texture at binding {}", binding);
    Ok(WBindGroup { inner: bind_group })
}

/// Create a bind group with texture and sampler (common case for sampled textures)
#[wasm_bindgen(js_name = createBindGroupWithTextureSampler)]
pub fn create_bind_group_with_texture_sampler(
    device: &WDevice,
    layout: &WBindGroupLayout,
    texture_binding: u32,
    texture: &WTexture,
    sampler_binding: u32,
    sampler: &WSampler,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let tex = texture
        .inner()
        .ok_or_else(|| JsValue::from_str("Cannot bind surface texture"))?;

    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: texture_binding,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(sampler.inner()),
            },
        ],
    });

    log::debug!(
        "Created bind group with texture at {} and sampler at {}",
        texture_binding, sampler_binding
    );
    Ok(WBindGroup { inner: bind_group })
}

/// Create a bind group with 2 buffers + 1 texture view + 1 sampler
#[wasm_bindgen(js_name = createBindGroupWith2BuffersTextureViewSampler)]
pub fn create_bind_group_with_2_buffers_texture_view_sampler(
    device: &WDevice,
    layout: &WBindGroupLayout,
    buffer0_binding: u32,
    buffer0: &WBuffer,
    buffer0_offset: u64,
    buffer0_size: u64,
    buffer1_binding: u32,
    buffer1: &WBuffer,
    buffer1_offset: u64,
    buffer1_size: u64,
    texture_binding: u32,
    texture_view: &WTextureView,
    sampler_binding: u32,
    sampler: &WSampler,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let view = texture_view
        .inner()
        .ok_or_else(|| JsValue::from_str("Cannot bind surface texture view"))?;

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: buffer0_binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer0.inner(),
                    offset: buffer0_offset,
                    size: std::num::NonZeroU64::new(buffer0_size),
                }),
            },
            wgpu::BindGroupEntry {
                binding: buffer1_binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer1.inner(),
                    offset: buffer1_offset,
                    size: std::num::NonZeroU64::new(buffer1_size),
                }),
            },
            wgpu::BindGroupEntry {
                binding: texture_binding,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(sampler.inner()),
            },
        ],
    });

    log::debug!("Created bind group with 2 buffers, texture view, and sampler");
    Ok(WBindGroup { inner: bind_group })
}

/// Create a bind group with 1 buffer + 1 sampler + 6 texture views
#[wasm_bindgen(js_name = createBindGroupWithBufferSampler6TextureViews)]
#[allow(clippy::too_many_arguments)]
pub fn create_bind_group_with_buffer_sampler_6_texture_views(
    device: &WDevice,
    layout: &WBindGroupLayout,
    buffer_binding: u32,
    buffer: &WBuffer,
    buffer_offset: u64,
    buffer_size: u64,
    sampler_binding: u32,
    sampler: &WSampler,
    tex0_binding: u32,
    tex0: &WTextureView,
    tex1_binding: u32,
    tex1: &WTextureView,
    tex2_binding: u32,
    tex2: &WTextureView,
    tex3_binding: u32,
    tex3: &WTextureView,
    tex4_binding: u32,
    tex4: &WTextureView,
    tex5_binding: u32,
    tex5: &WTextureView,
) -> Result<WBindGroup, JsValue> {
    let state = device.state();
    let state = state.borrow();

    let view0 = tex0.inner().ok_or_else(|| JsValue::from_str("tex0 cannot be surface"))?;
    let view1 = tex1.inner().ok_or_else(|| JsValue::from_str("tex1 cannot be surface"))?;
    let view2 = tex2.inner().ok_or_else(|| JsValue::from_str("tex2 cannot be surface"))?;
    let view3 = tex3.inner().ok_or_else(|| JsValue::from_str("tex3 cannot be surface"))?;
    let view4 = tex4.inner().ok_or_else(|| JsValue::from_str("tex4 cannot be surface"))?;
    let view5 = tex5.inner().ok_or_else(|| JsValue::from_str("tex5 cannot be surface"))?;

    let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout.inner,
        entries: &[
            wgpu::BindGroupEntry {
                binding: buffer_binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buffer.inner(),
                    offset: buffer_offset,
                    size: std::num::NonZeroU64::new(buffer_size),
                }),
            },
            wgpu::BindGroupEntry {
                binding: sampler_binding,
                resource: wgpu::BindingResource::Sampler(sampler.inner()),
            },
            wgpu::BindGroupEntry {
                binding: tex0_binding,
                resource: wgpu::BindingResource::TextureView(view0),
            },
            wgpu::BindGroupEntry {
                binding: tex1_binding,
                resource: wgpu::BindingResource::TextureView(view1),
            },
            wgpu::BindGroupEntry {
                binding: tex2_binding,
                resource: wgpu::BindingResource::TextureView(view2),
            },
            wgpu::BindGroupEntry {
                binding: tex3_binding,
                resource: wgpu::BindingResource::TextureView(view3),
            },
            wgpu::BindGroupEntry {
                binding: tex4_binding,
                resource: wgpu::BindingResource::TextureView(view4),
            },
            wgpu::BindGroupEntry {
                binding: tex5_binding,
                resource: wgpu::BindingResource::TextureView(view5),
            },
        ],
    });

    log::debug!("Created bind group with buffer, sampler, and 6 texture views");
    Ok(WBindGroup { inner: bind_group })
}
