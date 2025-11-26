//! Command encoding and render pass

use super::device::GlContextRef;
use super::pipeline::WRenderPipeline;
use super::types::WLoadOp;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Render pass encoder - equivalent to GPURenderPassEncoder
/// In WebGL, we execute commands immediately rather than recording them
#[wasm_bindgen]
pub struct WRenderPassEncoder {
    context: GlContextRef,
    current_pipeline: Option<glow::Program>,
    current_vao: Option<glow::VertexArray>,
    current_topology: u32,
}

impl WRenderPassEncoder {
    fn new(context: GlContextRef) -> Self {
        Self {
            context,
            current_pipeline: None,
            current_vao: None,
            current_topology: glow::TRIANGLES,
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
        }
        self.current_pipeline = Some(pipeline.program);
        self.current_vao = Some(pipeline.vao);
        self.current_topology = pipeline.topology.to_gl();
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
            let offset = (first_index * 2) as i32; // assuming u16 indices
            if instance_count > 1 {
                ctx.gl.draw_elements_instanced(
                    self.current_topology,
                    index_count as i32,
                    glow::UNSIGNED_SHORT,
                    offset,
                    instance_count as i32,
                );
            } else {
                ctx.gl.draw_elements(
                    self.current_topology,
                    index_count as i32,
                    glow::UNSIGNED_SHORT,
                    offset,
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
    /// Begin a render pass
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
