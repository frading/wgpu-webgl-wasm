//! Render pipeline creation and management

use super::device::GlContextRef;
use super::shader::WShaderModule;
use super::types::{WPrimitiveTopology, WVertexFormat, WBlendState, WBlendFactor, WBlendOperation, WBlendComponent};
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Stored vertex attribute for later configuration
#[derive(Clone)]
pub struct StoredVertexAttribute {
    pub location: u32,
    pub offset: u32,
    pub format: WVertexFormat,
}

/// Stored vertex buffer layout for later configuration
#[derive(Clone)]
pub struct StoredVertexBufferLayout {
    pub stride: u32,
    pub attributes: Vec<StoredVertexAttribute>,
}

/// Render pipeline - equivalent to GPURenderPipeline
#[wasm_bindgen]
pub struct WRenderPipeline {
    context: GlContextRef,
    pub(crate) program: glow::Program,
    pub(crate) vao: glow::VertexArray,
    pub(crate) topology: WPrimitiveTopology,
    /// Stored vertex layouts for configuring attributes when buffers are bound
    /// Index corresponds to the vertex buffer slot
    pub(crate) vertex_layouts: Vec<StoredVertexBufferLayout>,
    // Rasterization state
    pub(crate) cull_mode: WCullMode,
    pub(crate) front_face: WFrontFace,
    // Depth state
    pub(crate) depth_test_enabled: bool,
    pub(crate) depth_write_enabled: bool,
    pub(crate) depth_compare: WCompareFunction,
    // Blend state
    pub(crate) blend_state: Option<WBlendState>,
}

impl Drop for WRenderPipeline {
    fn drop(&mut self) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.delete_program(self.program);
            ctx.gl.delete_vertex_array(self.vao);
        }
        log::debug!("Render pipeline destroyed");
    }
}

impl WRenderPipeline {
    pub fn context(&self) -> GlContextRef {
        self.context.clone()
    }
}

#[wasm_bindgen]
impl WRenderPipeline {
    /// Get bind group layout at the specified index
    /// In WebGL, this returns an empty layout since we don't have explicit bind group layouts
    /// tied to pipelines. The layout is inferred from shader reflection at bind time.
    #[wasm_bindgen(js_name = getBindGroupLayout)]
    pub fn get_bind_group_layout(&self, _index: u32) -> super::bind_group::WBindGroupLayout {
        // In WebGL, bind group layouts are not explicitly tied to pipelines
        // We return an empty layout that can be used for compatibility
        log::debug!("getBindGroupLayout called for index {}", _index);
        super::bind_group::WBindGroupLayout {
            entries: Vec::new(),
        }
    }
}

/// Setup uniform block bindings for a linked program.
/// Naga generates uniform block names like `CameraUniforms_block_0Vertex` with
/// variables inside named `_group_1_binding_0_vs`.
/// We need to bind these to the correct binding points so that bind_buffer_range works correctly.
unsafe fn setup_uniform_block_bindings(gl: &glow::Context, program: glow::Program) {
    // Get the number of uniform blocks from the program
    let num_uniform_blocks = gl.get_program_parameter_i32(program, glow::ACTIVE_UNIFORM_BLOCKS) as u32;
    log::info!("Program has {} uniform blocks", num_uniform_blocks);

    for block_index in 0..num_uniform_blocks {
        let block_name = gl.get_active_uniform_block_name(program, block_index);
        let block_size = gl.get_active_uniform_block_parameter_i32(
            program,
            block_index,
            glow::UNIFORM_BLOCK_DATA_SIZE,
        );

        log::info!(
            "Uniform block {}: name='{}', size={}",
            block_index, block_name, block_size
        );

        // Get the number of uniforms in this block
        let num_uniforms_in_block = gl.get_active_uniform_block_parameter_i32(
            program,
            block_index,
            glow::UNIFORM_BLOCK_ACTIVE_UNIFORMS,
        ) as usize;

        // Try to find group/binding info from the first uniform in this block
        let mut found_binding = false;
        if num_uniforms_in_block > 0 {
            // Get the uniform indices for this block
            let mut uniform_indices = vec![0i32; num_uniforms_in_block];
            gl.get_active_uniform_block_parameter_i32_slice(
                program,
                block_index,
                glow::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
                &mut uniform_indices,
            );

            // Check the first uniform's name for group/binding info
            if let Some(&first_uniform_index) = uniform_indices.first() {
                if first_uniform_index >= 0 {
                    if let Some(uniform) = gl.get_active_uniform(program, first_uniform_index as u32) {
                        log::info!("First uniform in block {}: '{}'", block_index, uniform.name);
                        if let Some((group, binding)) = parse_group_binding_from_name(&uniform.name) {
                            // Use group as binding point (binding within group is usually 0)
                            let binding_point = group;
                            gl.uniform_block_binding(program, block_index, binding_point);
                            log::info!(
                                "Bound uniform block '{}' (index {}) to binding point {} (group={}, binding={})",
                                block_name, block_index, binding_point, group, binding
                            );
                            found_binding = true;
                        }
                    }
                }
            }
        }

        if !found_binding {
            // Fallback: try to parse from block name
            if let Some(binding) = parse_binding_from_block_name(&block_name) {
                gl.uniform_block_binding(program, block_index, binding);
                log::info!(
                    "Bound uniform block '{}' (index {}) to binding point {} (from block name)",
                    block_name, block_index, binding
                );
            } else {
                // Last resort: use block index
                gl.uniform_block_binding(program, block_index, block_index);
                log::warn!(
                    "Could not parse binding from '{}', using block index {} as binding point",
                    block_name, block_index
                );
            }
        }
    }
}

/// Parse group and binding from a uniform name like "_group_1_binding_0_vs.worldPos"
fn parse_group_binding_from_name(name: &str) -> Option<(u32, u32)> {
    if let Some(group_pos) = name.find("_group_") {
        let after_group = &name[group_pos + 7..];
        let group_str: String = after_group.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(group) = group_str.parse::<u32>() {
            if let Some(binding_pos) = after_group.find("_binding_") {
                let after_binding = &after_group[binding_pos + 9..];
                let binding_str: String = after_binding.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(binding) = binding_str.parse::<u32>() {
                    return Some((group, binding));
                }
            }
        }
    }
    None
}

/// Parse the group and binding index from a Naga-generated uniform block name.
///
/// Naga generates block names like "CameraUniforms_block_0Vertex" or "ObjectUniforms_block_1Vertex"
/// where the number after "block_" is a sequential index.
///
/// The actual binding info is in the variable name inside the block: "_group_1_binding_0_vs"
///
/// For now, we parse the block name format: "{Name}_block_{N}Vertex" or "{Name}_block_{N}Fragment"
/// and treat N as a sequential index. We need to query the uniform inside to get the real binding.
///
/// Alternative approach: parse "_group{G}_binding{B}" format if present anywhere in the name.
fn parse_binding_from_block_name(name: &str) -> Option<u32> {
    // First try the new Naga format: look for "_group_X_binding_Y" pattern
    if let Some(group_pos) = name.find("_group_") {
        let after_group = &name[group_pos + 7..]; // Skip "_group_"
        // Extract group number
        let group_str: String = after_group.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(group) = group_str.parse::<u32>() {
            // Now look for "_binding_"
            if let Some(binding_pos) = after_group.find("_binding_") {
                let after_binding = &after_group[binding_pos + 9..]; // Skip "_binding_"
                let binding_str: String = after_binding.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(binding) = binding_str.parse::<u32>() {
                    // In WebGL, we flatten group+binding into a single binding point
                    // Common approach: binding_point = group * MAX_BINDINGS_PER_GROUP + binding
                    // But simpler: just use sequential binding points based on block index
                    // For now, return the binding from the first group we encounter
                    log::info!("Parsed group={}, binding={} from '{}'", group, binding, name);
                    return Some(binding);
                }
            }
        }
    }

    // Try old format: "_binding" followed by number (without underscore before number)
    if let Some(binding_pos) = name.find("_binding") {
        let after_binding = &name[binding_pos + 8..]; // Skip "_binding"
        let binding_str: String = after_binding.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(binding) = binding_str.parse::<u32>() {
            return Some(binding);
        }
    }

    // Try parsing "block_N" format as fallback
    if let Some(block_pos) = name.find("_block_") {
        let after_block = &name[block_pos + 7..]; // Skip "_block_"
        let block_str: String = after_block.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(block_idx) = block_str.parse::<u32>() {
            log::info!("Parsed block index {} from '{}'", block_idx, name);
            return Some(block_idx);
        }
    }

    None
}

/// Create a render pipeline (simple version without vertex attributes)
/// This links shaders into a program and sets up the vertex array object
#[wasm_bindgen(js_name = createRenderPipeline)]
pub fn create_render_pipeline(
    device: &super::WDevice,
    shader_module: &WShaderModule,
    topology: WPrimitiveTopology,
) -> Result<WRenderPipeline, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        // Create program and link shaders
        let program = ctx
            .gl
            .create_program()
            .map_err(|e| JsValue::from_str(&format!("Failed to create program: {}", e)))?;

        if let Some(vs) = shader_module.vertex_shader {
            ctx.gl.attach_shader(program, vs);
        }
        if let Some(fs) = shader_module.fragment_shader {
            ctx.gl.attach_shader(program, fs);
        }

        ctx.gl.link_program(program);

        if !ctx.gl.get_program_link_status(program) {
            let log = ctx.gl.get_program_info_log(program);
            ctx.gl.delete_program(program);
            return Err(JsValue::from_str(&format!(
                "Program linking failed: {}",
                log
            )));
        }

        // Setup uniform block bindings after linking
        setup_uniform_block_bindings(&ctx.gl, program);

        // Create VAO (required for WebGL2)
        let vao = ctx
            .gl
            .create_vertex_array()
            .map_err(|e| JsValue::from_str(&format!("Failed to create VAO: {}", e)))?;

        log::info!("Render pipeline created successfully");

        Ok(WRenderPipeline {
            context: context.clone(),
            program,
            vao,
            topology,
            vertex_layouts: Vec::new(),
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
            blend_state: None,
        })
    }
}

/// Vertex attribute description for pipeline creation
#[wasm_bindgen]
pub struct WVertexAttribute {
    /// Shader location (e.g., 0 for @location(0))
    pub location: u32,
    /// Byte offset within the vertex
    pub offset: u32,
    /// Format of the attribute
    pub format: WVertexFormat,
}

#[wasm_bindgen]
impl WVertexAttribute {
    #[wasm_bindgen(constructor)]
    pub fn new(location: u32, offset: u32, format: WVertexFormat) -> Self {
        Self { location, offset, format }
    }
}

/// Vertex buffer layout description
#[wasm_bindgen]
pub struct WVertexBufferLayout {
    /// Stride in bytes between consecutive vertices
    pub stride: u32,
    /// Attributes in this buffer
    attributes: Vec<WVertexAttribute>,
}

#[wasm_bindgen]
impl WVertexBufferLayout {
    #[wasm_bindgen(constructor)]
    pub fn new(stride: u32) -> Self {
        Self { stride, attributes: Vec::new() }
    }

    /// Add an attribute to this buffer layout
    #[wasm_bindgen(js_name = addAttribute)]
    pub fn add_attribute(&mut self, location: u32, offset: u32, format: WVertexFormat) {
        self.attributes.push(WVertexAttribute { location, offset, format });
    }
}

/// Cull mode for rasterization
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WCullMode {
    None = 0,
    Front = 1,
    Back = 2,
}

impl WCullMode {
    pub fn to_gl(self) -> Option<u32> {
        match self {
            WCullMode::None => None,
            WCullMode::Front => Some(glow::FRONT),
            WCullMode::Back => Some(glow::BACK),
        }
    }
}

/// Front face winding order
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WFrontFace {
    Ccw = 0,
    Cw = 1,
}

impl WFrontFace {
    pub fn to_gl(self) -> u32 {
        match self {
            WFrontFace::Ccw => glow::CCW,
            WFrontFace::Cw => glow::CW,
        }
    }
}

/// Compare function for depth/stencil
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WCompareFunction {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

impl WCompareFunction {
    pub fn to_gl(self) -> u32 {
        match self {
            WCompareFunction::Never => glow::NEVER,
            WCompareFunction::Less => glow::LESS,
            WCompareFunction::Equal => glow::EQUAL,
            WCompareFunction::LessEqual => glow::LEQUAL,
            WCompareFunction::Greater => glow::GREATER,
            WCompareFunction::NotEqual => glow::NOTEQUAL,
            WCompareFunction::GreaterEqual => glow::GEQUAL,
            WCompareFunction::Always => glow::ALWAYS,
        }
    }
}

/// Extended render pipeline with more state
#[wasm_bindgen]
pub struct WRenderPipelineDescriptor {
    // Primitive state
    topology: WPrimitiveTopology,
    cull_mode: WCullMode,
    front_face: WFrontFace,
    // Depth state
    depth_test_enabled: bool,
    depth_write_enabled: bool,
    depth_compare: WCompareFunction,
    // Vertex layouts (up to 4)
    vertex_layouts: Vec<StoredVertexBufferLayout>,
    // Blend state
    blend_state: Option<WBlendState>,
}

#[wasm_bindgen]
impl WRenderPipelineDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new(topology: WPrimitiveTopology) -> Self {
        Self {
            topology,
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
            vertex_layouts: Vec::new(),
            blend_state: None,
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
    pub fn set_depth_test(&mut self, enabled: bool, write_enabled: bool, compare: WCompareFunction) {
        self.depth_test_enabled = enabled;
        self.depth_write_enabled = write_enabled;
        self.depth_compare = compare;
    }

    /// Set blend state for the color attachment
    #[wasm_bindgen(js_name = setBlendState)]
    pub fn set_blend_state(
        &mut self,
        color_op: WBlendOperation, color_src: WBlendFactor, color_dst: WBlendFactor,
        alpha_op: WBlendOperation, alpha_src: WBlendFactor, alpha_dst: WBlendFactor,
    ) {
        self.blend_state = Some(WBlendState {
            color: WBlendComponent { operation: color_op, src_factor: color_src, dst_factor: color_dst },
            alpha: WBlendComponent { operation: alpha_op, src_factor: alpha_src, dst_factor: alpha_dst },
        });
        log::info!("Set blend state: color({:?}, {:?}, {:?}), alpha({:?}, {:?}, {:?})",
            color_op, color_src, color_dst, alpha_op, alpha_src, alpha_dst);
    }

    #[wasm_bindgen(js_name = addVertexBufferLayout)]
    pub fn add_vertex_buffer_layout(&mut self, stride: u32) -> usize {
        let index = self.vertex_layouts.len();
        self.vertex_layouts.push(StoredVertexBufferLayout {
            stride,
            attributes: Vec::new(),
        });
        index
    }

    #[wasm_bindgen(js_name = addVertexAttribute)]
    pub fn add_vertex_attribute(&mut self, buffer_index: usize, location: u32, offset: u32, format: WVertexFormat) {
        if buffer_index < self.vertex_layouts.len() {
            self.vertex_layouts[buffer_index].attributes.push(StoredVertexAttribute {
                location,
                offset,
                format,
            });
        }
    }
}

/// Create a render pipeline with full descriptor
#[wasm_bindgen(js_name = createRenderPipelineFromDescriptor)]
pub fn create_render_pipeline_from_descriptor(
    device: &super::WDevice,
    shader_module: &WShaderModule,
    descriptor: &WRenderPipelineDescriptor,
) -> Result<WRenderPipeline, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        // Create program and link shaders
        let program = ctx
            .gl
            .create_program()
            .map_err(|e| JsValue::from_str(&format!("Failed to create program: {}", e)))?;

        if let Some(vs) = shader_module.vertex_shader {
            ctx.gl.attach_shader(program, vs);
        }
        if let Some(fs) = shader_module.fragment_shader {
            ctx.gl.attach_shader(program, fs);
        }

        ctx.gl.link_program(program);

        if !ctx.gl.get_program_link_status(program) {
            let log = ctx.gl.get_program_info_log(program);
            ctx.gl.delete_program(program);
            return Err(JsValue::from_str(&format!(
                "Program linking failed: {}",
                log
            )));
        }

        // Setup uniform block bindings after linking
        setup_uniform_block_bindings(&ctx.gl, program);

        // Create VAO
        let vao = ctx
            .gl
            .create_vertex_array()
            .map_err(|e| JsValue::from_str(&format!("Failed to create VAO: {}", e)))?;

        log::info!("Render pipeline created with {} vertex buffer layouts, blend={:?}",
            descriptor.vertex_layouts.len(), descriptor.blend_state.is_some());

        // Log details about each vertex layout
        for (i, layout) in descriptor.vertex_layouts.iter().enumerate() {
            log::info!("  Layout {}: stride={}, {} attributes", i, layout.stride, layout.attributes.len());
            for attr in &layout.attributes {
                log::info!("    Attribute location={}, offset={}, format={:?}", attr.location, attr.offset, attr.format);
            }
        }

        Ok(WRenderPipeline {
            context: context.clone(),
            program,
            vao,
            topology: descriptor.topology,
            vertex_layouts: descriptor.vertex_layouts.clone(),
            cull_mode: descriptor.cull_mode,
            front_face: descriptor.front_face,
            depth_test_enabled: descriptor.depth_test_enabled,
            depth_write_enabled: descriptor.depth_write_enabled,
            depth_compare: descriptor.depth_compare,
            blend_state: descriptor.blend_state,
        })
    }
}

/// Create a render pipeline with vertex buffer layout
/// This version allows specifying vertex attributes from a buffer
#[wasm_bindgen(js_name = createRenderPipelineWithLayout)]
pub fn create_render_pipeline_with_layout(
    device: &super::WDevice,
    shader_module: &WShaderModule,
    topology: WPrimitiveTopology,
    vertex_layout: &WVertexBufferLayout,
) -> Result<WRenderPipeline, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        // Create program and link shaders
        let program = ctx
            .gl
            .create_program()
            .map_err(|e| JsValue::from_str(&format!("Failed to create program: {}", e)))?;

        if let Some(vs) = shader_module.vertex_shader {
            ctx.gl.attach_shader(program, vs);
        }
        if let Some(fs) = shader_module.fragment_shader {
            ctx.gl.attach_shader(program, fs);
        }

        ctx.gl.link_program(program);

        if !ctx.gl.get_program_link_status(program) {
            let log = ctx.gl.get_program_info_log(program);
            ctx.gl.delete_program(program);
            return Err(JsValue::from_str(&format!(
                "Program linking failed: {}",
                log
            )));
        }

        // Setup uniform block bindings after linking
        setup_uniform_block_bindings(&ctx.gl, program);

        // Create VAO (required for WebGL2)
        let vao = ctx
            .gl
            .create_vertex_array()
            .map_err(|e| JsValue::from_str(&format!("Failed to create VAO: {}", e)))?;

        // Store the vertex layout for later use when setVertexBuffer is called
        // In WebGL, glVertexAttribPointer captures the currently bound buffer,
        // so we can't configure attributes until the buffer is bound.
        let stored_layout = StoredVertexBufferLayout {
            stride: vertex_layout.stride,
            attributes: vertex_layout.attributes.iter().map(|attr| {
                StoredVertexAttribute {
                    location: attr.location,
                    offset: attr.offset,
                    format: attr.format,
                }
            }).collect(),
        };

        log::info!("Render pipeline with vertex layout created successfully");

        Ok(WRenderPipeline {
            context: context.clone(),
            program,
            vao,
            topology,
            vertex_layouts: vec![stored_layout],
            cull_mode: WCullMode::None,
            front_face: WFrontFace::Ccw,
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: WCompareFunction::Less,
            blend_state: None,
        })
    }
}
