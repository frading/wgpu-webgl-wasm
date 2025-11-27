//! Command encoding and render pass

use super::bind_group::WBindGroup;
use super::buffer::WBuffer;
use super::device::GlContextRef;
use super::pipeline::{WRenderPipeline, StoredVertexBufferLayout};
use super::texture::WTextureView;
use super::types::WLoadOp;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Index format for draw_indexed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexFormat {
    Uint16 = 0,
    Uint32 = 1,
}

impl IndexFormat {
    fn to_gl(self) -> u32 {
        match self {
            IndexFormat::Uint16 => glow::UNSIGNED_SHORT,
            IndexFormat::Uint32 => glow::UNSIGNED_INT,
        }
    }

    fn byte_size(self) -> u32 {
        match self {
            IndexFormat::Uint16 => 2,
            IndexFormat::Uint32 => 4,
        }
    }
}

/// Render pass encoder - equivalent to GPURenderPassEncoder
/// In WebGL, we execute commands immediately rather than recording them
#[wasm_bindgen]
pub struct WRenderPassEncoder {
    context: GlContextRef,
    current_pipeline: Option<glow::Program>,
    current_vao: Option<glow::VertexArray>,
    current_topology: u32,
    /// Stored vertex layouts from the current pipeline for configuring attributes
    /// Index corresponds to vertex buffer slot
    current_vertex_layouts: Vec<StoredVertexBufferLayout>,
    /// Current index buffer format
    current_index_format: IndexFormat,
}

impl WRenderPassEncoder {
    fn new(context: GlContextRef) -> Self {
        Self {
            context,
            current_pipeline: None,
            current_vao: None,
            current_topology: glow::TRIANGLES,
            current_vertex_layouts: Vec::new(),
            current_index_format: IndexFormat::Uint16,
        }
    }
}

#[wasm_bindgen]
impl WRenderPassEncoder {
    /// Set the render pipeline
    #[wasm_bindgen(js_name = setPipeline)]
    pub fn set_pipeline(&mut self, pipeline: &WRenderPipeline) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.use_program(Some(pipeline.program));
            ctx.gl.bind_vertex_array(Some(pipeline.vao));

            // Apply depth state
            if pipeline.depth_test_enabled {
                ctx.gl.enable(glow::DEPTH_TEST);
                ctx.gl.depth_func(pipeline.depth_compare.to_gl());
                ctx.gl.depth_mask(pipeline.depth_write_enabled);
                log::info!("Depth test enabled: compare={:?}, write={}",
                    pipeline.depth_compare, pipeline.depth_write_enabled);
            } else {
                log::info!(">> Depth test NOT enabled");
                ctx.gl.disable(glow::DEPTH_TEST);
            }

            // Apply blend state
            if let Some(ref blend) = pipeline.blend_state {
                if blend.is_enabled() {
                    ctx.gl.enable(glow::BLEND);
                    ctx.gl.blend_func_separate(
                        blend.color.src_factor.to_gl(),
                        blend.color.dst_factor.to_gl(),
                        blend.alpha.src_factor.to_gl(),
                        blend.alpha.dst_factor.to_gl(),
                    );
                    ctx.gl.blend_equation_separate(
                        blend.color.operation.to_gl(),
                        blend.alpha.operation.to_gl(),
                    );
                    log::debug!("Blend enabled: src={:?}, dst={:?}",
                        blend.color.src_factor, blend.color.dst_factor);
                } else {
                    ctx.gl.disable(glow::BLEND);
                }
            } else {
                ctx.gl.disable(glow::BLEND);
            }
        }
        self.current_pipeline = Some(pipeline.program);
        self.current_vao = Some(pipeline.vao);
        self.current_topology = pipeline.topology.to_gl();
        // Store all vertex layouts for use when setVertexBuffer is called
        self.current_vertex_layouts = pipeline.vertex_layouts.clone();
    }

    /// Draw primitives
    /// vertex_count: number of vertices to draw
    /// instance_count: number of instances (1 for non-instanced)
    /// first_vertex: offset to first vertex
    /// first_instance: offset to first instance (usually 0)
    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        _first_instance: u32,
    ) {
        let ctx = self.context.borrow();
        unsafe {
            if instance_count > 1 {
                ctx.gl.draw_arrays_instanced(
                    self.current_topology,
                    first_vertex as i32,
                    vertex_count as i32,
                    instance_count as i32,
                );
            } else {
                ctx.gl.draw_arrays(
                    self.current_topology,
                    first_vertex as i32,
                    vertex_count as i32,
                );
            }
        }
    }

    /// Draw indexed primitives
    #[wasm_bindgen(js_name = drawIndexed)]
    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        _base_vertex: i32,
        _first_instance: u32,
    ) {
        let ctx = self.context.borrow();
        unsafe {
            let index_type = self.current_index_format.to_gl();
            let byte_offset = (first_index * self.current_index_format.byte_size()) as i32;

            if instance_count > 1 {
                ctx.gl.draw_elements_instanced(
                    self.current_topology,
                    index_count as i32,
                    index_type,
                    byte_offset,
                    instance_count as i32,
                );
            } else {
                ctx.gl.draw_elements(
                    self.current_topology,
                    index_count as i32,
                    index_type,
                    byte_offset,
                );
            }
        }
    }

    /// Set the viewport
    #[wasm_bindgen(js_name = setViewport)]
    pub fn set_viewport(&self, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.viewport(x as i32, y as i32, width as i32, height as i32);
            ctx.gl.depth_range_f32(min_depth, max_depth);
        }
    }

    /// Set scissor rect
    #[wasm_bindgen(js_name = setScissorRect)]
    pub fn set_scissor_rect(&self, x: u32, y: u32, width: u32, height: u32) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.enable(glow::SCISSOR_TEST);
            ctx.gl.scissor(x as i32, y as i32, width as i32, height as i32);
        }
    }

    /// Set a vertex buffer for a specific slot
    /// slot: the vertex buffer slot index
    /// buffer: the buffer to bind
    /// offset: byte offset into the buffer
    #[wasm_bindgen(js_name = setVertexBuffer)]
    pub fn set_vertex_buffer(&self, slot: u32, buffer: &WBuffer, offset: u32) {
        let ctx = self.context.borrow();
        unsafe {
            // Bind the buffer
            ctx.gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer.raw));

            // Configure vertex attributes now that the buffer is bound
            // In WebGL, glVertexAttribPointer captures the currently bound GL_ARRAY_BUFFER
            // Look up the layout for this specific slot
            if let Some(layout) = self.current_vertex_layouts.get(slot as usize) {
                for attr in &layout.attributes {
                    ctx.gl.enable_vertex_attrib_array(attr.location);
                    ctx.gl.vertex_attrib_pointer_f32(
                        attr.location,
                        attr.format.components(),
                        attr.format.gl_type(),
                        false, // normalized
                        layout.stride as i32,
                        (attr.offset + offset) as i32,
                    );
                    log::debug!(
                        "Configured vertex attribute {} for slot {}: offset={}, components={}, stride={}",
                        attr.location, slot, attr.offset + offset, attr.format.components(), layout.stride
                    );
                }
            } else {
                log::warn!("No vertex layout found for slot {}", slot);
            }

            log::debug!("Vertex buffer set at slot {}, offset {}", slot, offset);
        }
    }

    /// Set the index buffer
    /// buffer: the index buffer to bind
    /// format: index format (0 = uint16, 1 = uint32)
    /// offset: byte offset into the buffer
    #[wasm_bindgen(js_name = setIndexBuffer)]
    pub fn set_index_buffer(&mut self, buffer: &WBuffer, format: u32, offset: u32) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(buffer.raw));
        }

        // Store the index format for draw_indexed
        self.current_index_format = if format == 1 {
            IndexFormat::Uint32
        } else {
            IndexFormat::Uint16
        };

        log::debug!("Index buffer set, format {:?}, offset {}", self.current_index_format, offset);
        let _ = offset; // Offset is handled in draw_indexed via first_index
    }

    /// Set a bind group at the given index
    ///
    /// group_index: the bind group slot (0-3 typically)
    /// bind_group: the bind group to set
    /// dynamic_offsets: optional dynamic offsets (not yet supported)
    #[wasm_bindgen(js_name = setBindGroup)]
    pub fn set_bind_group(&self, group_index: u32, bind_group: &WBindGroup) {
        let ctx = self.context.borrow();

        // Apply the bind group's bindings to GL state
        // Pass group_index so uniform buffers are bound to the correct binding point
        // Also pass the current program so we can set sampler uniforms
        bind_group.apply_with_program(&ctx.gl, group_index, self.current_pipeline);

        log::debug!("Bind group {} set with {} entries",
            group_index, bind_group.entries.len());
    }

    /// End the render pass
    pub fn end(&self) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.bind_vertex_array(None);
            ctx.gl.use_program(None);
            ctx.gl.disable(glow::SCISSOR_TEST);
        }
        log::debug!("Render pass ended");
    }
}

/// Command encoder - equivalent to GPUCommandEncoder
/// In WebGL, we execute commands immediately, so this is mostly a pass-through
#[wasm_bindgen]
pub struct WCommandEncoder {
    context: GlContextRef,
}

#[wasm_bindgen]
impl WCommandEncoder {
    /// Begin a render pass (simple version without texture view)
    /// clear_r, clear_g, clear_b, clear_a: clear color (used if load_op is Clear)
    /// load_op: whether to clear or load existing content
    #[wasm_bindgen(js_name = beginRenderPass)]
    pub fn begin_render_pass(
        &self,
        clear_r: f32,
        clear_g: f32,
        clear_b: f32,
        clear_a: f32,
        load_op: WLoadOp,
    ) -> WRenderPassEncoder {
        let ctx = self.context.borrow();

        unsafe {
            // Bind default framebuffer
            ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            // Set viewport to canvas size
            ctx.gl.viewport(0, 0, ctx.width as i32, ctx.height as i32);

            // Clear if requested
            if load_op == WLoadOp::Clear {
                ctx.gl.clear_color(clear_r, clear_g, clear_b, clear_a);
                ctx.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
        }

        log::debug!("Render pass begun");
        WRenderPassEncoder::new(self.context.clone())
    }

    /// Begin a render pass with a color attachment texture view
    ///
    /// This is the full version that accepts a texture view as the render target.
    /// If the texture view is a surface texture (default framebuffer), we render
    /// directly to the canvas. Otherwise, we set up an FBO for render-to-texture.
    ///
    /// color_view: the texture view to render to
    /// clear_r, clear_g, clear_b, clear_a: clear color (used if load_op is Clear)
    /// load_op: whether to clear or load existing content
    #[wasm_bindgen(js_name = beginRenderPassWithView)]
    pub fn begin_render_pass_with_view(
        &self,
        color_view: &WTextureView,
        clear_r: f32,
        clear_g: f32,
        clear_b: f32,
        clear_a: f32,
        load_op: WLoadOp,
    ) -> WRenderPassEncoder {
        // Need mutable borrow for FBO cache
        let mut ctx = self.context.borrow_mut();

        unsafe {
            if color_view.is_surface() {
                // Render to default framebuffer (canvas)
                ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                ctx.gl.viewport(0, 0, ctx.width as i32, ctx.height as i32);
                log::debug!("Render pass targeting surface (default framebuffer)");
            } else if let Some(texture) = color_view.texture_raw {
                // Render to texture via FBO
                // We flip the viewport Y to account for OpenGL's bottom-left texture origin.
                // This makes the FBO content match WebGPU's top-left origin convention.
                // Get or create FBO for this texture
                let cached = if let Some(existing) = ctx.fbo_cache.get(&texture) {
                    existing.fbo
                } else {
                    // Create a new FBO
                    let fbo = ctx.gl.create_framebuffer()
                        .expect("Failed to create framebuffer");

                    // Bind and attach the texture
                    ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                    ctx.gl.framebuffer_texture_2d(
                        glow::FRAMEBUFFER,
                        glow::COLOR_ATTACHMENT0,
                        glow::TEXTURE_2D,
                        Some(texture),
                        color_view.base_mip_level as i32,
                    );

                    // Create and attach a depth renderbuffer
                    let depth_rb = ctx.gl.create_renderbuffer()
                        .expect("Failed to create depth renderbuffer");
                    ctx.gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth_rb));
                    ctx.gl.renderbuffer_storage(
                        glow::RENDERBUFFER,
                        glow::DEPTH_COMPONENT24,
                        color_view.width as i32,
                        color_view.height as i32,
                    );
                    ctx.gl.framebuffer_renderbuffer(
                        glow::FRAMEBUFFER,
                        glow::DEPTH_ATTACHMENT,
                        glow::RENDERBUFFER,
                        Some(depth_rb),
                    );
                    ctx.gl.bind_renderbuffer(glow::RENDERBUFFER, None);

                    // Check framebuffer completeness
                    let status = ctx.gl.check_framebuffer_status(glow::FRAMEBUFFER);
                    if status != glow::FRAMEBUFFER_COMPLETE {
                        let status_str = match status {
                            glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "INCOMPLETE_ATTACHMENT",
                            glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "INCOMPLETE_MISSING_ATTACHMENT",
                            glow::FRAMEBUFFER_INCOMPLETE_DIMENSIONS => "INCOMPLETE_DIMENSIONS",
                            glow::FRAMEBUFFER_UNSUPPORTED => "UNSUPPORTED",
                            _ => "UNKNOWN",
                        };
                        log::error!("Framebuffer incomplete: status={} ({})", status, status_str);
                    } else {
                        log::info!("Created FBO with depth for texture, {}x{}, mip_level={}",
                            color_view.width, color_view.height, color_view.base_mip_level);
                    }

                    // Cache the FBO with its depth renderbuffer
                    ctx.fbo_cache.insert(texture, super::device::CachedFbo {
                        fbo,
                        depth_renderbuffer: depth_rb,
                        width: color_view.width,
                        height: color_view.height,
                    });
                    fbo
                };

                // Bind the FBO
                ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(cached));
                ctx.gl.viewport(0, 0, color_view.width as i32, color_view.height as i32);
                log::debug!("Render pass targeting texture via FBO ({}x{})", color_view.width, color_view.height);
            } else {
                // No texture and not surface - shouldn't happen, fallback to default
                log::warn!("TextureView has no texture and is not surface, using default framebuffer");
                ctx.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                ctx.gl.viewport(0, 0, ctx.width as i32, ctx.height as i32);
            }

            // Clear if requested
            if load_op == WLoadOp::Clear {
                ctx.gl.clear_color(clear_r, clear_g, clear_b, clear_a);
                ctx.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
                log::info!("Cleared with color ({}, {}, {}, {})", clear_r, clear_g, clear_b, clear_a);
            }
        }

        log::info!("Render pass begun with view, is_surface={}", color_view.is_surface());
        WRenderPassEncoder::new(self.context.clone())
    }

    /// Finish encoding and return (in WebGL this is a no-op since commands execute immediately)
    pub fn finish(&self) {
        log::debug!("Command encoder finished");
    }
}

/// Create a command encoder
#[wasm_bindgen(js_name = createCommandEncoder)]
pub fn create_command_encoder(device: &super::WDevice) -> WCommandEncoder {
    WCommandEncoder {
        context: device.context(),
    }
}
