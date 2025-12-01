//! Render pipeline wrapper

use wasm_bindgen::prelude::*;
use super::device::WDevice;
use super::shader::WShaderModule;
use super::bind_group::{WBindGroupLayout, WPipelineLayout};
use super::types::{
    WPrimitiveTopology, WVertexFormat, WCullMode, WFrontFace,
    WBlendFactor, WBlendOperation, WVertexBufferLayout,
};
use super::texture::WTextureFormat;
use super::sampler::WCompareFunction;

/// Render pipeline
#[wasm_bindgen]
pub struct WRenderPipeline {
    pub(crate) inner: wgpu::RenderPipeline,
}

impl WRenderPipeline {
    pub(crate) fn inner(&self) -> &wgpu::RenderPipeline {
        &self.inner
    }
}

#[wasm_bindgen]
impl WRenderPipeline {
    /// Get bind group layout at index (for auto-generated layouts)
    #[wasm_bindgen(js_name = getBindGroupLayout)]
    pub fn get_bind_group_layout(&self, index: u32) -> WBindGroupLayout {
        log::info!("getBindGroupLayout called with index={}", index);
        let layout = self.inner.get_bind_group_layout(index);
        log::info!("getBindGroupLayout returned layout for index={}", index);
        WBindGroupLayout {
            inner: layout,
            entry_count: 0, // We don't know the entry count from auto-generated layouts
        }
    }
}

/// Render pipeline descriptor (builder pattern)
#[wasm_bindgen]
pub struct WRenderPipelineDescriptor {
    topology: WPrimitiveTopology,
    cull_mode: WCullMode,
    front_face: WFrontFace,
    depth_test_enabled: bool,
    depth_write_enabled: bool,
    depth_compare: WCompareFunction,
    depth_format: Option<WTextureFormat>,
    color_format: WTextureFormat,
    vertex_layouts: Vec<VertexBufferLayoutData>,
    blend_enabled: bool,
    blend_color_src: WBlendFactor,
    blend_color_dst: WBlendFactor,
    blend_color_op: WBlendOperation,
    blend_alpha_src: WBlendFactor,
    blend_alpha_dst: WBlendFactor,
    blend_alpha_op: WBlendOperation,
    vertex_entry_point: String,
    fragment_entry_point: String,
}

struct VertexBufferLayoutData {
    stride: u64,
    step_mode: wgpu::VertexStepMode,
    attributes: Vec<wgpu::VertexAttribute>,
}

#[wasm_bindgen]
impl WRenderPipelineDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new(topology: WPrimitiveTopology, vertex_entry_point: &str, fragment_entry_point: &str) -> Self {
        Self {
            topology,
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
            depth_format: None,
            color_format: WTextureFormat::Bgra8Unorm,
            vertex_layouts: Vec::new(),
            blend_enabled: false,
            blend_color_src: WBlendFactor::One,
            blend_color_dst: WBlendFactor::Zero,
            blend_color_op: WBlendOperation::Add,
            blend_alpha_src: WBlendFactor::One,
            blend_alpha_dst: WBlendFactor::Zero,
            blend_alpha_op: WBlendOperation::Add,
            vertex_entry_point: vertex_entry_point.to_string(),
            fragment_entry_point: fragment_entry_point.to_string(),
        }
    }

    #[wasm_bindgen(js_name = setCullMode)]
    pub fn set_cull_mode(&mut self, cull_mode: WCullMode) {
        self.cull_mode = cull_mode;
    }

    #[wasm_bindgen(js_name = setFrontFace)]
    pub fn set_front_face(&mut self, front_face: WFrontFace) {
        self.front_face = front_face;
    }

    #[wasm_bindgen(js_name = setDepthTest)]
    pub fn set_depth_test(
        &mut self,
        enabled: bool,
        write_enabled: bool,
        compare: WCompareFunction,
    ) {
        self.depth_test_enabled = enabled;
        self.depth_write_enabled = write_enabled;
        self.depth_compare = compare;
    }

    #[wasm_bindgen(js_name = setDepthFormat)]
    pub fn set_depth_format(&mut self, format: WTextureFormat) {
        self.depth_format = Some(format);
    }

    #[wasm_bindgen(js_name = setColorFormat)]
    pub fn set_color_format(&mut self, format: WTextureFormat) {
        self.color_format = format;
    }

    #[wasm_bindgen(js_name = setBlendState)]
    pub fn set_blend_state(
        &mut self,
        color_op: WBlendOperation,
        color_src: WBlendFactor,
        color_dst: WBlendFactor,
        alpha_op: WBlendOperation,
        alpha_src: WBlendFactor,
        alpha_dst: WBlendFactor,
    ) {
        self.blend_enabled = true;
        self.blend_color_op = color_op;
        self.blend_color_src = color_src;
        self.blend_color_dst = color_dst;
        self.blend_alpha_op = alpha_op;
        self.blend_alpha_src = alpha_src;
        self.blend_alpha_dst = alpha_dst;
    }

    #[wasm_bindgen(js_name = addVertexBufferLayout)]
    pub fn add_vertex_buffer_layout(&mut self, stride: u64, step_mode: u32) -> usize {
        let index = self.vertex_layouts.len();
        self.vertex_layouts.push(VertexBufferLayoutData {
            stride,
            step_mode: if step_mode == 1 {
                wgpu::VertexStepMode::Instance
            } else {
                wgpu::VertexStepMode::Vertex
            },
            attributes: Vec::new(),
        });
        index
    }

    #[wasm_bindgen(js_name = addVertexAttribute)]
    pub fn add_vertex_attribute(
        &mut self,
        buffer_index: usize,
        location: u32,
        offset: u64,
        format: WVertexFormat,
    ) {
        if buffer_index < self.vertex_layouts.len() {
            self.vertex_layouts[buffer_index]
                .attributes
                .push(wgpu::VertexAttribute {
                    format: format.to_wgpu(),
                    offset,
                    shader_location: location,
                });
        }
    }
}

/// Create a render pipeline with vertex buffer layout
// #[wasm_bindgen(js_name = createRenderPipelineWithLayout)]
// pub fn create_render_pipeline_with_layout(
//     device: &WDevice,
//     shader_module: &WShaderModule,
//     topology: WPrimitiveTopology,
//     vertex_layout: &WVertexBufferLayout,
// ) -> Result<WRenderPipeline, JsValue> {
//     let state = device.state();
//     let state = state.borrow();

//     // Convert WVertexBufferLayout to wgpu format
//     let attributes: Vec<wgpu::VertexAttribute> = vertex_layout
//         .attributes
//         .iter()
//         .map(|attr| wgpu::VertexAttribute {
//             format: attr.format.to_wgpu(),
//             offset: attr.offset as u64,
//             shader_location: attr.location,
//         })
//         .collect();

//     let vertex_buffer_layout = wgpu::VertexBufferLayout {
//         array_stride: vertex_layout.stride as u64,
//         step_mode: wgpu::VertexStepMode::Vertex,
//         attributes: &attributes,
//     };

//     let pipeline = state
//         .device
//         .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: None,
//             layout: None,
//             vertex: wgpu::VertexState {
//                 module: shader_module.inner(),
//                 entry_point: Some("vertex"),
//                 buffers: &[vertex_buffer_layout],
//                 compilation_options: Default::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: shader_module.inner(),
//                 entry_point: Some("fragment"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: state.surface_config.format,
//                     blend: None,
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: topology.to_wgpu(),
//                 ..Default::default()
//             },
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//     log::debug!("Created render pipeline with vertex layout");

//     Ok(WRenderPipeline { inner: pipeline })
// }

/// Create a render pipeline (simple version without vertex attributes)
// #[wasm_bindgen(js_name = createRenderPipeline)]
// pub fn create_render_pipeline(
//     device: &WDevice,
//     shader_module: &WShaderModule,
//     topology: WPrimitiveTopology,
// ) -> Result<WRenderPipeline, JsValue> {
//     let state = device.state();
//     let state = state.borrow();

//     let pipeline = state
//         .device
//         .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: None,
//             layout: None,
//             vertex: wgpu::VertexState {
//                 module: shader_module.inner(),
//                 entry_point: Some("vertex"),
//                 buffers: &[],
//                 compilation_options: Default::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: shader_module.inner(),
//                 entry_point: Some("fragment"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: state.surface_config.format,
//                     blend: None,
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: topology.to_wgpu(),
//                 ..Default::default()
//             },
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//     log::debug!("Created render pipeline");

//     Ok(WRenderPipeline { inner: pipeline })
// }

// /// Create a render pipeline from descriptor
// #[wasm_bindgen(js_name = createRenderPipelineFromDescriptor)]
// pub fn create_render_pipeline_from_descriptor(
//     device: &WDevice,
//     shader_module: &WShaderModule,
//     descriptor: &WRenderPipelineDescriptor,
// ) -> Result<WRenderPipeline, JsValue> {
//     let state = device.state();
//     let state = state.borrow();

//     log::info!(
//         "createRenderPipelineFromDescriptor: topology={:?}, cull={:?}, front={:?}, depth_test={}, depth_write={}, blend={}, descriptor_color_format={:?}, surface_format={:?}, vertex_layouts={}",
//         descriptor.topology, descriptor.cull_mode, descriptor.front_face,
//         descriptor.depth_test_enabled, descriptor.depth_write_enabled,
//         descriptor.blend_enabled, descriptor.color_format, state.surface_config.format,
//         descriptor.vertex_layouts.len()
//     );

//     // Build vertex buffer layouts
//     let vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout> = descriptor
//         .vertex_layouts
//         .iter()
//         .map(|layout| wgpu::VertexBufferLayout {
//             array_stride: layout.stride,
//             step_mode: layout.step_mode,
//             attributes: &layout.attributes,
//         })
//         .collect();

//     // Build blend state
//     let blend = if descriptor.blend_enabled {
//         Some(wgpu::BlendState {
//             color: wgpu::BlendComponent {
//                 operation: descriptor.blend_color_op.to_wgpu(),
//                 src_factor: descriptor.blend_color_src.to_wgpu(),
//                 dst_factor: descriptor.blend_color_dst.to_wgpu(),
//             },
//             alpha: wgpu::BlendComponent {
//                 operation: descriptor.blend_alpha_op.to_wgpu(),
//                 src_factor: descriptor.blend_alpha_src.to_wgpu(),
//                 dst_factor: descriptor.blend_alpha_dst.to_wgpu(),
//             },
//         })
//     } else {
//         None
//     };

//     // Build depth stencil state
//     let depth_stencil = if descriptor.depth_test_enabled {
//         Some(wgpu::DepthStencilState {
//             format: descriptor
//                 .depth_format
//                 .unwrap_or(WTextureFormat::Depth24Plus)
//                 .to_wgpu(),
//             depth_write_enabled: descriptor.depth_write_enabled,
//             depth_compare: descriptor.depth_compare.to_wgpu(),
//             stencil: wgpu::StencilState::default(),
//             bias: wgpu::DepthBiasState::default(),
//         })
//     } else {
//         None
//     };

//     // Use the color format from the descriptor
//     let color_format = descriptor.color_format.to_wgpu();

//     log::info!("Creating pipeline with color format {:?}", color_format);

//     let pipeline = state
//         .device
//         .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: None,
//             layout: None,
//             vertex: wgpu::VertexState {
//                 module: shader_module.inner(),
//                 entry_point: Some("vertex"),
//                 buffers: &vertex_buffer_layouts,
//                 compilation_options: Default::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: shader_module.inner(),
//                 entry_point: Some("fragment"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: color_format,
//                     blend,
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: Default::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: descriptor.topology.to_wgpu(),
//                 front_face: descriptor.front_face.to_wgpu(),
//                 cull_mode: descriptor.cull_mode.to_wgpu(),
//                 ..Default::default()
//             },
//             depth_stencil,
//             multisample: wgpu::MultisampleState::default(),
//             multiview_mask: None,
//             cache: None,
//         });

//     log::debug!("Created render pipeline from descriptor");

//     Ok(WRenderPipeline { inner: pipeline })
// }

/// Create a render pipeline from descriptor with explicit pipeline layout
#[wasm_bindgen(js_name = createRenderPipelineWithPipelineLayout)]
pub fn create_render_pipeline_with_pipeline_layout(
    device: &WDevice,
    shader_module: &WShaderModule,
    descriptor: &WRenderPipelineDescriptor,
    pipeline_layout: &WPipelineLayout,
) -> Result<WRenderPipeline, JsValue> {
    let state = device.state();
    let state = state.borrow();

    log::info!(
        "createRenderPipelineWithPipelineLayout: topology={:?}, cull={:?}, front={:?}, depth_test={}, depth_write={}, blend={}, vertex_layouts={}",
        descriptor.topology, descriptor.cull_mode, descriptor.front_face,
        descriptor.depth_test_enabled, descriptor.depth_write_enabled,
        descriptor.blend_enabled,
        descriptor.vertex_layouts.len()
    );

    // Build vertex buffer layouts
    let vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout> = descriptor
        .vertex_layouts
        .iter()
        .map(|layout| wgpu::VertexBufferLayout {
            array_stride: layout.stride,
            step_mode: layout.step_mode,
            attributes: &layout.attributes,
        })
        .collect();

    // Build blend state
    let blend = if descriptor.blend_enabled {
        Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                operation: descriptor.blend_color_op.to_wgpu(),
                src_factor: descriptor.blend_color_src.to_wgpu(),
                dst_factor: descriptor.blend_color_dst.to_wgpu(),
            },
            alpha: wgpu::BlendComponent {
                operation: descriptor.blend_alpha_op.to_wgpu(),
                src_factor: descriptor.blend_alpha_src.to_wgpu(),
                dst_factor: descriptor.blend_alpha_dst.to_wgpu(),
            },
        })
    } else {
        None
    };

    // Build depth stencil state
    let depth_stencil = if descriptor.depth_test_enabled {
        Some(wgpu::DepthStencilState {
            format: descriptor
                .depth_format
                .unwrap_or(WTextureFormat::Depth24Plus)
                .to_wgpu(),
            depth_write_enabled: descriptor.depth_write_enabled,
            depth_compare: descriptor.depth_compare.to_wgpu(),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
    } else {
        None
    };

    // Use the color format from the descriptor
    let color_format = descriptor.color_format.to_wgpu();

    log::info!(
        "Creating pipeline with explicit layout, color format {:?}, vertex_entry={}, fragment_entry={}",
        color_format, descriptor.vertex_entry_point, descriptor.fragment_entry_point
    );

    let pipeline = state
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout.inner()),
            vertex: wgpu::VertexState {
                module: shader_module.inner(),
                entry_point: Some(&descriptor.vertex_entry_point),
                buffers: &vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module.inner(),
                entry_point: Some(&descriptor.fragment_entry_point),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: descriptor.topology.to_wgpu(),
                front_face: descriptor.front_face.to_wgpu(),
                cull_mode: descriptor.cull_mode.to_wgpu(),
                ..Default::default()
            },
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

    log::debug!("Created render pipeline with explicit pipeline layout");

    Ok(WRenderPipeline { inner: pipeline })
}
