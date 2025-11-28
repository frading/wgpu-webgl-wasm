//! Bind group and bind group layout management
//!
//! In WebGL, there's no direct equivalent to WebGPU's bind groups.
//! We emulate them by tracking which resources (buffers, textures, samplers)
//! are bound to which binding slots, then applying them when drawing.

use super::buffer::WBuffer;
use super::device::GlContextRef;
use super::sampler::WSampler;
use super::texture::{WTexture, WTextureView};
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Binding type enum
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WBindingType {
    /// Uniform buffer
    UniformBuffer = 0,
    /// Storage buffer (read-only)
    StorageBuffer = 1,
    /// Storage buffer (read-write)
    StorageBufferReadWrite = 2,
    /// Sampler
    Sampler = 3,
    /// Sampled texture
    SampledTexture = 4,
    /// Storage texture
    StorageTexture = 5,
}

/// A single entry in a bind group layout
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WBindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: u32, // Shader stage flags
    pub binding_type: WBindingType,
    // For buffers
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

/// Bind group layout - describes the structure of a bind group
#[wasm_bindgen]
pub struct WBindGroupLayout {
    pub(crate) entries: Vec<WBindGroupLayoutEntry>,
}

#[wasm_bindgen]
impl WBindGroupLayout {
    /// Get the number of entries in this layout
    #[wasm_bindgen(getter, js_name = entryCount)]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// A resource bound in a bind group entry
#[derive(Clone)]
pub(crate) enum BoundResource {
    Buffer {
        buffer: glow::Buffer,
        offset: u64,
        size: u64,
    },
    Texture {
        texture: glow::Texture,
        target: u32, // GL_TEXTURE_2D, GL_TEXTURE_2D_ARRAY, etc.
    },
    Sampler {
        sampler: glow::Sampler,
    },
    /// Combined texture and sampler (common in WebGL where they're often paired)
    TextureSampler {
        texture: glow::Texture,
        sampler: glow::Sampler,
        target: u32,
    },
}

/// A single entry in a bind group (the actual bound resource)
pub(crate) struct BindGroupEntry {
    pub binding: u32,
    pub resource: BoundResource,
}

/// Bind group - a collection of resources bound together
#[wasm_bindgen]
pub struct WBindGroup {
    pub(crate) layout: Vec<WBindGroupLayoutEntry>,
    pub(crate) entries: Vec<BindGroupEntry>,
    pub(crate) context: GlContextRef,
}

impl WBindGroup {
    /// Apply this bind group's bindings to the GL state with program for sampler uniforms
    ///
    /// This binds uniform buffers to their respective binding points,
    /// and textures/samplers to texture units. For textures, it also sets the
    /// sampler uniform in the shader to point to the correct texture unit.
    ///
    /// group_index: The bind group index (from setBindGroup). In WebGL, we use this
    /// as the uniform buffer binding point since WebGL doesn't have bind groups.
    /// The shader's uniform blocks are bound to binding points matching the group index.
    ///
    /// program: The currently bound program, used to set sampler uniforms.
    pub(crate) fn apply_with_program(&self, gl: &glow::Context, group_index: u32, program: Option<glow::Program>) {
        for entry in &self.entries {
            match &entry.resource {
                BoundResource::Buffer { buffer, offset, size } => {
                    // Find the layout entry to determine the binding type
                    let layout_entry = self.layout.iter().find(|e| e.binding == entry.binding);

                    if let Some(layout) = layout_entry {
                        match layout.binding_type {
                            WBindingType::UniformBuffer => {
                                // Use group_index as the binding point
                                // This matches how we set up uniform block bindings in the shader
                                log::info!(
                                    "Binding uniform buffer: group={}, binding={}, offset={}, size={}",
                                    group_index, entry.binding, offset, size
                                );
                                unsafe {
                                    gl.bind_buffer_range(
                                        glow::UNIFORM_BUFFER,
                                        group_index, // Use group index as binding point
                                        Some(*buffer),
                                        *offset as i32,
                                        *size as i32,
                                    );
                                }
                            }
                            WBindingType::StorageBuffer | WBindingType::StorageBufferReadWrite => {
                                // WebGL2 doesn't have SSBOs, but we can try with transform feedback
                                // or just log a warning for now
                                log::warn!(
                                    "Storage buffers not fully supported in WebGL2, binding {} ignored",
                                    entry.binding
                                );
                            }
                            _ => {}
                        }
                    } else {
                        // No layout entry found, assume uniform buffer
                        log::info!(
                            "Binding uniform buffer (no layout): group={}, binding={}, offset={}, size={}",
                            group_index, entry.binding, offset, size
                        );
                        unsafe {
                            gl.bind_buffer_range(
                                glow::UNIFORM_BUFFER,
                                group_index, // Use group index as binding point
                                Some(*buffer),
                                *offset as i32,
                                *size as i32,
                            );
                        }
                    }
                }
                BoundResource::Sampler { sampler } => {
                    unsafe {
                        gl.bind_sampler(entry.binding, Some(*sampler));
                    }
                    log::debug!("Bound sampler to texture unit {}", entry.binding);
                }
                BoundResource::Texture { texture, target } => {
                    let texture_unit = entry.binding;
                    unsafe {
                        gl.active_texture(glow::TEXTURE0 + texture_unit);
                        gl.bind_texture(*target, Some(*texture));

                        // Set sampler uniform if we have a program
                        if let Some(prog) = program {
                            // Try to find the sampler uniform for this binding
                            // Naga generates names like "_group_0_binding_0_fs" for fragment samplers
                            let sampler_names = [
                                format!("_group_{}_binding_{}_fs", group_index, entry.binding),
                                format!("_group_{}_binding_{}_vs", group_index, entry.binding),
                            ];

                            for name in &sampler_names {
                                if let Some(location) = gl.get_uniform_location(prog, name) {
                                    gl.uniform_1_i32(Some(&location), texture_unit as i32);
                                    log::info!("Set sampler uniform '{}' to texture unit {}", name, texture_unit);
                                    break;
                                }
                            }
                        }
                    }
                    log::info!("Bound texture {:?} to texture unit {}", texture, texture_unit);
                }
                BoundResource::TextureSampler { texture, sampler, target } => {
                    unsafe {
                        gl.active_texture(glow::TEXTURE0 + entry.binding);
                        gl.bind_texture(*target, Some(*texture));
                        gl.bind_sampler(entry.binding, Some(*sampler));
                    }
                    log::debug!("Bound texture+sampler to texture unit {}", entry.binding);
                }
            }
        }
    }
}

// JavaScript-friendly API for creating bind groups
// These functions accept JS values since wasm-bindgen can't directly pass Vec<T>

/// Create a bind group layout from JavaScript
///
/// entries_json should be a JSON array of entry objects:
/// [{ binding: 0, visibility: 1, type: "uniform-buffer" }, ...]
#[wasm_bindgen(js_name = createBindGroupLayout)]
pub fn create_bind_group_layout_from_js(
    _device: &super::WDevice,
    entries_js: JsValue,
) -> Result<WBindGroupLayout, JsValue> {
    let entries_array: js_sys::Array = entries_js.dyn_into()
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

        // Determine binding type from the entry
        let binding_type = if js_sys::Reflect::has(&entry_obj, &"buffer".into()).unwrap_or(false) {
            let buffer_obj = js_sys::Reflect::get(&entry_obj, &"buffer".into())?;
            let type_str = js_sys::Reflect::get(&buffer_obj, &"type".into())
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "uniform".to_string());

            match type_str.as_str() {
                "storage" => WBindingType::StorageBuffer,
                "read-only-storage" => WBindingType::StorageBuffer,
                _ => WBindingType::UniformBuffer,
            }
        } else if js_sys::Reflect::has(&entry_obj, &"sampler".into()).unwrap_or(false) {
            WBindingType::Sampler
        } else if js_sys::Reflect::has(&entry_obj, &"texture".into()).unwrap_or(false) {
            WBindingType::SampledTexture
        } else if js_sys::Reflect::has(&entry_obj, &"storageTexture".into()).unwrap_or(false) {
            WBindingType::StorageTexture
        } else {
            WBindingType::UniformBuffer // Default
        };

        entries.push(WBindGroupLayoutEntry {
            binding,
            visibility,
            binding_type,
            has_dynamic_offset: false,
            min_binding_size: 0,
        });
    }

    log::debug!("Created bind group layout with {} entries", entries.len());

    Ok(WBindGroupLayout { entries })
}

/// Create a bind group with a single buffer binding
///
/// This is a simple API for the common case of binding a single uniform buffer.
#[wasm_bindgen(js_name = createBindGroupWithBuffer)]
pub fn create_bind_group_with_buffer(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    buffer: &WBuffer,
    offset: u64,
    size: u64,
) -> WBindGroup {
    let entries = vec![BindGroupEntry {
        binding,
        resource: BoundResource::Buffer {
            buffer: buffer.raw,
            offset,
            size,
        },
    }];

    log::debug!("Created bind group with buffer at binding {}", binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with two buffer bindings
#[wasm_bindgen(js_name = createBindGroupWith2Buffers)]
pub fn create_bind_group_with_2_buffers(
    device: &super::WDevice,
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
    let entries = vec![
        BindGroupEntry {
            binding: binding0,
            resource: BoundResource::Buffer {
                buffer: buffer0.raw,
                offset: offset0,
                size: size0,
            },
        },
        BindGroupEntry {
            binding: binding1,
            resource: BoundResource::Buffer {
                buffer: buffer1.raw,
                offset: offset1,
                size: size1,
            },
        },
    ];

    log::debug!("Created bind group with 2 buffer bindings");

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Pipeline layout - collection of bind group layouts
/// In WebGL this is mostly metadata, but we store the layouts for validation
#[wasm_bindgen]
pub struct WPipelineLayout {
    pub(crate) bind_group_layout_count: u32,
    // We don't actually store references to the layouts since they're managed by JS heap
    // This is just a marker type for the pipeline
}

#[wasm_bindgen]
impl WPipelineLayout {
    #[wasm_bindgen(getter, js_name = bindGroupLayoutCount)]
    pub fn bind_group_layout_count(&self) -> u32 {
        self.bind_group_layout_count
    }
}

/// Create a pipeline layout
/// In WebGL, this is mostly a no-op since we don't have explicit pipeline layouts.
/// We just track the number of bind group layouts for validation.
#[wasm_bindgen(js_name = createPipelineLayout)]
pub fn create_pipeline_layout(
    _device: &super::WDevice,
    bind_group_layout_count: u32,
) -> WPipelineLayout {
    log::debug!("Created pipeline layout with {} bind group layouts", bind_group_layout_count);
    WPipelineLayout {
        bind_group_layout_count,
    }
}

/// Create a bind group with three buffer bindings
#[wasm_bindgen(js_name = createBindGroupWith3Buffers)]
pub fn create_bind_group_with_3_buffers(
    device: &super::WDevice,
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
    let entries = vec![
        BindGroupEntry {
            binding: binding0,
            resource: BoundResource::Buffer {
                buffer: buffer0.raw,
                offset: offset0,
                size: size0,
            },
        },
        BindGroupEntry {
            binding: binding1,
            resource: BoundResource::Buffer {
                buffer: buffer1.raw,
                offset: offset1,
                size: size1,
            },
        },
        BindGroupEntry {
            binding: binding2,
            resource: BoundResource::Buffer {
                buffer: buffer2.raw,
                offset: offset2,
                size: size2,
            },
        },
    ];

    log::debug!("Created bind group with 3 buffer bindings");

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with a sampler binding only
#[wasm_bindgen(js_name = createBindGroupWithSampler)]
pub fn create_bind_group_with_sampler(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    sampler: &WSampler,
) -> WBindGroup {
    let entries = vec![BindGroupEntry {
        binding,
        resource: BoundResource::Sampler {
            sampler: sampler.raw,
        },
    }];

    log::debug!("Created bind group with sampler at binding {}", binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with a texture binding only
#[wasm_bindgen(js_name = createBindGroupWithTexture)]
pub fn create_bind_group_with_texture(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    texture: &WTexture,
) -> WBindGroup {
    let entries = if let Some(tex) = texture.raw {
        vec![BindGroupEntry {
            binding,
            resource: BoundResource::Texture {
                texture: tex,
                target: glow::TEXTURE_2D,
            },
        }]
    } else {
        log::warn!("Cannot bind surface texture as regular texture, creating empty binding");
        Vec::new()
    };

    log::debug!("Created bind group with texture at binding {}", binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with texture and sampler (common case for sampled textures)
#[wasm_bindgen(js_name = createBindGroupWithTextureSampler)]
pub fn create_bind_group_with_texture_sampler(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    texture_binding: u32,
    texture: &WTexture,
    sampler_binding: u32,
    sampler: &WSampler,
) -> WBindGroup {
    let mut entries = Vec::new();

    // Add texture entry
    if let Some(tex) = texture.raw {
        entries.push(BindGroupEntry {
            binding: texture_binding,
            resource: BoundResource::Texture {
                texture: tex,
                target: glow::TEXTURE_2D,
            },
        });
    }

    // Add sampler entry
    entries.push(BindGroupEntry {
        binding: sampler_binding,
        resource: BoundResource::Sampler {
            sampler: sampler.raw,
        },
    });

    log::debug!("Created bind group with texture at {} and sampler at {}",
        texture_binding, sampler_binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create an empty bind group (for bind groups with only texture/sampler from views)
#[wasm_bindgen(js_name = createEmptyBindGroup)]
pub fn create_empty_bind_group(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
) -> WBindGroup {
    log::debug!("Created empty bind group");

    WBindGroup {
        layout: layout.entries.clone(),
        entries: Vec::new(),
        context: device.context(),
    }
}

/// Create a bind group with a texture view binding
#[wasm_bindgen(js_name = createBindGroupWithTextureView)]
pub fn create_bind_group_with_texture_view(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    binding: u32,
    texture_view: &WTextureView,
) -> WBindGroup {
    let entries = if let Some(tex) = texture_view.raw() {
        vec![BindGroupEntry {
            binding,
            resource: BoundResource::Texture {
                texture: tex,
                target: glow::TEXTURE_2D,
            },
        }]
    } else {
        log::warn!("Cannot bind surface texture view as regular texture, creating empty binding");
        Vec::new()
    };

    log::debug!("Created bind group with texture view at binding {}", binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with texture view and sampler (common case for sampled textures)
#[wasm_bindgen(js_name = createBindGroupWithTextureViewSampler)]
pub fn create_bind_group_with_texture_view_sampler(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    texture_binding: u32,
    texture_view: &WTextureView,
    sampler_binding: u32,
    sampler: &WSampler,
) -> WBindGroup {
    let mut entries = Vec::new();

    // Add texture entry from view
    if let Some(tex) = texture_view.raw() {
        entries.push(BindGroupEntry {
            binding: texture_binding,
            resource: BoundResource::Texture {
                texture: tex,
                target: glow::TEXTURE_2D,
            },
        });
    }

    // Add sampler entry
    entries.push(BindGroupEntry {
        binding: sampler_binding,
        resource: BoundResource::Sampler {
            sampler: sampler.raw,
        },
    });

    log::debug!("Created bind group with texture view at {} and sampler at {}",
        texture_binding, sampler_binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with 2 buffers + 1 texture view + 1 sampler
/// Common case for materials with uniform buffers and a sampled texture
#[wasm_bindgen(js_name = createBindGroupWith2BuffersTextureViewSampler)]
pub fn create_bind_group_with_2_buffers_texture_view_sampler(
    device: &super::WDevice,
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
) -> WBindGroup {
    let mut entries = Vec::new();

    // Add buffer entries
    entries.push(BindGroupEntry {
        binding: buffer0_binding,
        resource: BoundResource::Buffer {
            buffer: buffer0.raw,
            offset: buffer0_offset,
            size: buffer0_size,
        },
    });

    entries.push(BindGroupEntry {
        binding: buffer1_binding,
        resource: BoundResource::Buffer {
            buffer: buffer1.raw,
            offset: buffer1_offset,
            size: buffer1_size,
        },
    });

    // Add texture entry from view
    if let Some(tex) = texture_view.raw() {
        entries.push(BindGroupEntry {
            binding: texture_binding,
            resource: BoundResource::Texture {
                texture: tex,
                target: glow::TEXTURE_2D,
            },
        });
    }

    // Add sampler entry
    entries.push(BindGroupEntry {
        binding: sampler_binding,
        resource: BoundResource::Sampler {
            sampler: sampler.raw,
        },
    });

    log::debug!("Created bind group with 2 buffers at {}/{}, texture view at {}, sampler at {}",
        buffer0_binding, buffer1_binding, texture_binding, sampler_binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}

/// Create a bind group with 1 buffer + 1 texture view + 1 sampler
/// Common case for materials with a uniform buffer and a sampled texture
#[wasm_bindgen(js_name = createBindGroupWithBufferTextureViewSampler)]
pub fn create_bind_group_with_buffer_texture_view_sampler(
    device: &super::WDevice,
    layout: &WBindGroupLayout,
    buffer_binding: u32,
    buffer: &WBuffer,
    buffer_offset: u64,
    buffer_size: u64,
    texture_binding: u32,
    texture_view: &WTextureView,
    sampler_binding: u32,
    sampler: &WSampler,
) -> WBindGroup {
    let mut entries = Vec::new();

    // Add buffer entry
    entries.push(BindGroupEntry {
        binding: buffer_binding,
        resource: BoundResource::Buffer {
            buffer: buffer.raw,
            offset: buffer_offset,
            size: buffer_size,
        },
    });

    // Add texture entry from view
    if let Some(tex) = texture_view.raw() {
        entries.push(BindGroupEntry {
            binding: texture_binding,
            resource: BoundResource::Texture {
                texture: tex,
                target: glow::TEXTURE_2D,
            },
        });
    }

    // Add sampler entry
    entries.push(BindGroupEntry {
        binding: sampler_binding,
        resource: BoundResource::Sampler {
            sampler: sampler.raw,
        },
    });

    log::debug!("Created bind group with buffer at {}, texture view at {}, sampler at {}",
        buffer_binding, texture_binding, sampler_binding);

    WBindGroup {
        layout: layout.entries.clone(),
        entries,
        context: device.context(),
    }
}
