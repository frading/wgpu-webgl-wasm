//! Bind group and bind group layout wrappers

use wasm_bindgen::prelude::*;
use super::device::WDevice;
use super::buffer::WBuffer;
use super::texture::WTextureView;
use super::sampler::WSampler;
use super::stats::{BIND_GROUP_COUNT, BIND_GROUP_LAYOUT_COUNT, PIPELINE_LAYOUT_COUNT};

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

        WBindGroup::new(bind_group)
    }
}

/// Bind group layout
#[wasm_bindgen]
pub struct WBindGroupLayout {
    pub(crate) inner: wgpu::BindGroupLayout,
    pub(crate) entry_count: u32,
}

impl WBindGroupLayout {
    pub(crate) fn new(inner: wgpu::BindGroupLayout, entry_count: u32) -> Self {
        BIND_GROUP_LAYOUT_COUNT.fetch_add(1, Ordering::Relaxed);
        Self { inner, entry_count }
    }
}

impl Drop for WBindGroupLayout {
    fn drop(&mut self) {
        BIND_GROUP_LAYOUT_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
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

    pub(crate) fn new(inner: wgpu::PipelineLayout, bind_group_layout_count: u32) -> Self {
        PIPELINE_LAYOUT_COUNT.fetch_add(1, Ordering::Relaxed);
        Self { inner, bind_group_layout_count }
    }
}

impl Drop for WPipelineLayout {
    fn drop(&mut self) {
        PIPELINE_LAYOUT_COUNT.fetch_sub(1, Ordering::Relaxed);
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

    pub(crate) fn new(inner: wgpu::BindGroup) -> Self {
        BIND_GROUP_COUNT.fetch_add(1, Ordering::Relaxed);
        Self { inner }
    }
}

impl Drop for WBindGroup {
    fn drop(&mut self) {
        BIND_GROUP_COUNT.fetch_sub(1, Ordering::Relaxed);
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

        let has_buffer = buffer_val.as_ref().map(|v| v.is_object()).unwrap_or(false);
        let has_sampler = sampler_val.as_ref().map(|v| v.is_object()).unwrap_or(false);
        let has_texture = texture_val.as_ref().map(|v| v.is_object()).unwrap_or(false);

        log::info!("createBindGroupLayout entry {}: binding={}, has_buffer={}, has_sampler={}, has_texture={}",
            i, binding, has_buffer, has_sampler, has_texture);

        let ty = if has_buffer {
            let buffer_obj = buffer_val.as_ref().unwrap();

            // Read minBindingSize from the buffer object
            let min_binding_size = js_sys::Reflect::get(buffer_obj, &"minBindingSize".into())
                .ok()
                .and_then(|v| v.as_f64())
                .and_then(|size| std::num::NonZeroU64::new(size as u64));

            // Read hasDynamicOffset
            let has_dynamic_offset = js_sys::Reflect::get(buffer_obj, &"hasDynamicOffset".into())
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Read buffer type
            let buffer_type_str = js_sys::Reflect::get(buffer_obj, &"type".into())
                .ok()
                .and_then(|v| v.as_string());

            let buffer_type = match buffer_type_str.as_deref() {
                Some("storage") => wgpu::BufferBindingType::Storage { read_only: false },
                Some("read-only-storage") => wgpu::BufferBindingType::Storage { read_only: true },
                _ => wgpu::BufferBindingType::Uniform, // "uniform" or default
            };

            log::info!("  buffer type: {:?}, hasDynamicOffset: {}, minBindingSize: {:?}",
                buffer_type_str, has_dynamic_offset, min_binding_size);

            wgpu::BindingType::Buffer {
                ty: buffer_type,
                has_dynamic_offset,
                min_binding_size,
            }
        } else if has_sampler {
            // Read sampler type from the sampler object
            let sampler_obj = sampler_val.as_ref().unwrap();
            let sampler_type = js_sys::Reflect::get(sampler_obj, &"type".into())
                .ok()
                .and_then(|v| v.as_string());

            let binding_type = match sampler_type.as_deref() {
                Some("comparison") => wgpu::SamplerBindingType::Comparison,
                Some("non-filtering") => wgpu::SamplerBindingType::NonFiltering,
                _ => wgpu::SamplerBindingType::Filtering, // "filtering" or default
            };

            log::info!("  sampler type: {:?} -> {:?}", sampler_type, binding_type);
            wgpu::BindingType::Sampler(binding_type)
        } else if has_texture {
            // Read texture properties
            let texture_obj = texture_val.as_ref().unwrap();

            let sample_type_str = js_sys::Reflect::get(texture_obj, &"sampleType".into())
                .ok()
                .and_then(|v| v.as_string());

            let view_dimension_str = js_sys::Reflect::get(texture_obj, &"viewDimension".into())
                .ok()
                .and_then(|v| v.as_string());

            let multisampled = js_sys::Reflect::get(texture_obj, &"multisampled".into())
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let sample_type = match sample_type_str.as_deref() {
                Some("depth") => wgpu::TextureSampleType::Depth,
                Some("sint") => wgpu::TextureSampleType::Sint,
                Some("uint") => wgpu::TextureSampleType::Uint,
                Some("unfilterable-float") => wgpu::TextureSampleType::Float { filterable: false },
                _ => wgpu::TextureSampleType::Float { filterable: true }, // "float" or default
            };

            let view_dimension = match view_dimension_str.as_deref() {
                Some("1d") => wgpu::TextureViewDimension::D1,
                Some("2d-array") => wgpu::TextureViewDimension::D2Array,
                Some("cube") => wgpu::TextureViewDimension::Cube,
                Some("cube-array") => wgpu::TextureViewDimension::CubeArray,
                Some("3d") => wgpu::TextureViewDimension::D3,
                _ => wgpu::TextureViewDimension::D2, // "2d" or default
            };

            log::info!("  texture sampleType: {:?} -> {:?}, viewDimension: {:?} -> {:?}, multisampled: {}",
                sample_type_str, sample_type, view_dimension_str, view_dimension, multisampled);

            wgpu::BindingType::Texture {
                sample_type,
                view_dimension,
                multisampled,
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

    Ok(WBindGroupLayout::new(layout, entry_count))
}

/// Pipeline layout builder - accumulates bind group layouts then creates the pipeline layout
#[wasm_bindgen]
pub struct WPipelineLayoutBuilder {
    layouts: Vec<wgpu::BindGroupLayout>,
}

#[wasm_bindgen]
impl WPipelineLayoutBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WPipelineLayoutBuilder {
        WPipelineLayoutBuilder {
            layouts: Vec::new(),
        }
    }

    /// Add a bind group layout
    #[wasm_bindgen(js_name = addBindGroupLayout)]
    pub fn add_bind_group_layout(&mut self, layout: &WBindGroupLayout) {
        // Clone the inner layout since we need to own it
        // Note: wgpu::BindGroupLayout is internally reference-counted
        self.layouts.push(layout.inner.clone());
    }

    /// Build the pipeline layout
    #[wasm_bindgen]
    pub fn build(self, device: &WDevice) -> WPipelineLayout {
        let state = device.state();
        let state = state.borrow();

        let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> = self.layouts.iter().collect();

        let layout = state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_layout_refs,
                push_constant_ranges: &[],
            });

        log::info!("Created pipeline layout with {} bind group layouts", self.layouts.len());

        WPipelineLayout::new(layout, self.layouts.len() as u32)
    }
}

