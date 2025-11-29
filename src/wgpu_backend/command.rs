//! Command encoder and render pass wrappers
//!
//! This implementation uses a deferred command recording approach because wgpu's
//! RenderPass borrows the CommandEncoder, which is difficult to expose through wasm-bindgen.
//!
//! Commands are recorded into a Vec and then executed when the pass ends.

use wasm_bindgen::prelude::*;
use super::device::{WDevice, get_device_state, DeviceState};
use super::buffer::WBuffer;
use super::pipeline::WRenderPipeline;
use super::bind_group::WBindGroup;
use super::texture::WTextureView;
use super::types::*;
use std::sync::Arc;
use std::cell::RefCell;

/// Recorded render command
#[derive(Clone)]
enum RenderCommand {
    SetPipeline(wgpu::RenderPipeline),
    SetBindGroup {
        index: u32,
        bind_group: wgpu::BindGroup,
    },
    SetVertexBuffer {
        slot: u32,
        buffer: wgpu::Buffer,
        offset: u64,
    },
    SetIndexBuffer {
        buffer: wgpu::Buffer,
        format: wgpu::IndexFormat,
        offset: u64,
    },
    Draw {
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    DrawIndexed {
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    },
    SetViewport {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    },
    SetScissorRect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

/// Render pass configuration
struct RenderPassConfig {
    /// Target texture view (None means surface texture)
    color_view: Option<wgpu::TextureView>,
    /// Depth texture view
    depth_view: Option<wgpu::TextureView>,
    /// Clear color
    clear_color: wgpu::Color,
    /// Load operation for color
    color_load_op: wgpu::LoadOp<wgpu::Color>,
    /// Load operation for depth
    depth_load_op: wgpu::LoadOp<f32>,
    /// Whether to write depth
    depth_write: bool,
}

/// Command encoder
#[wasm_bindgen]
pub struct WCommandEncoder {
    device_state: Arc<RefCell<DeviceState>>,
    /// Recorded render passes, each with their config and commands
    render_passes: Vec<(RenderPassConfig, Vec<RenderCommand>)>,
}

/// Render pass encoder - records commands for later execution
#[wasm_bindgen]
pub struct WRenderPassEncoder {
    device_state: Arc<RefCell<DeviceState>>,
    config: RenderPassConfig,
    commands: Vec<RenderCommand>,
    encoder_index: usize,
}

/// Create a command encoder
#[wasm_bindgen(js_name = createCommandEncoder)]
pub fn create_command_encoder(device: &WDevice) -> WCommandEncoder {
    log::debug!("Creating command encoder");
    WCommandEncoder {
        device_state: device.state(),
        render_passes: Vec::new(),
    }
}

#[wasm_bindgen]
impl WCommandEncoder {
    /// Begin a render pass targeting the surface
    #[wasm_bindgen(js_name = beginRenderPass)]
    pub fn begin_render_pass(
        &mut self,
        clear_r: f32,
        clear_g: f32,
        clear_b: f32,
        clear_a: f32,
        load_op: WLoadOp,
    ) -> WRenderPassEncoder {
        log::debug!(
            "Begin render pass: clear=({}, {}, {}, {}), load_op={:?}",
            clear_r, clear_g, clear_b, clear_a, load_op
        );

        let clear_color = wgpu::Color {
            r: clear_r as f64,
            g: clear_g as f64,
            b: clear_b as f64,
            a: clear_a as f64,
        };

        let color_load_op = match load_op {
            WLoadOp::Clear => wgpu::LoadOp::Clear(clear_color),
            WLoadOp::Load => wgpu::LoadOp::Load,
        };

        let config = RenderPassConfig {
            color_view: None, // Will use surface texture
            depth_view: None,
            clear_color,
            color_load_op,
            depth_load_op: wgpu::LoadOp::Clear(1.0),
            depth_write: false,
        };

        let encoder_index = self.render_passes.len();

        WRenderPassEncoder {
            device_state: self.device_state.clone(),
            config,
            commands: Vec::new(),
            encoder_index,
        }
    }

    /// Begin a render pass with a texture view target
    #[wasm_bindgen(js_name = beginRenderPassWithView)]
    pub fn begin_render_pass_with_view(
        &mut self,
        color_view: &WTextureView,
        clear_r: f32,
        clear_g: f32,
        clear_b: f32,
        clear_a: f32,
        load_op: WLoadOp,
    ) -> WRenderPassEncoder {
        log::debug!(
            "Begin render pass with view: is_surface={}, clear=({}, {}, {}, {})",
            color_view.is_surface_texture(),
            clear_r, clear_g, clear_b, clear_a
        );

        let clear_color = wgpu::Color {
            r: clear_r as f64,
            g: clear_g as f64,
            b: clear_b as f64,
            a: clear_a as f64,
        };

        let color_load_op = match load_op {
            WLoadOp::Clear => wgpu::LoadOp::Clear(clear_color),
            WLoadOp::Load => wgpu::LoadOp::Load,
        };

        // Clone the view if it's not a surface texture
        let color_view_inner = if color_view.is_surface_texture() {
            None
        } else {
            color_view.inner().cloned()
        };

        let config = RenderPassConfig {
            color_view: color_view_inner,
            depth_view: None,
            clear_color,
            color_load_op,
            depth_load_op: wgpu::LoadOp::Clear(1.0),
            depth_write: false,
        };

        let encoder_index = self.render_passes.len();

        WRenderPassEncoder {
            device_state: self.device_state.clone(),
            config,
            commands: Vec::new(),
            encoder_index,
        }
    }

    /// Begin a render pass with color and depth attachments
    #[wasm_bindgen(js_name = beginRenderPassWithDepth)]
    pub fn begin_render_pass_with_depth(
        &mut self,
        color_view: &WTextureView,
        depth_view: &WTextureView,
        clear_r: f32,
        clear_g: f32,
        clear_b: f32,
        clear_a: f32,
        load_op: WLoadOp,
        depth_clear_value: f32,
        depth_load_op: WLoadOp,
    ) -> WRenderPassEncoder {
        log::debug!(
            "Begin render pass with depth: is_surface={}, clear=({}, {}, {}, {}), depth_clear={}",
            color_view.is_surface_texture(),
            clear_r, clear_g, clear_b, clear_a,
            depth_clear_value
        );

        let clear_color = wgpu::Color {
            r: clear_r as f64,
            g: clear_g as f64,
            b: clear_b as f64,
            a: clear_a as f64,
        };

        let color_load_op = match load_op {
            WLoadOp::Clear => wgpu::LoadOp::Clear(clear_color),
            WLoadOp::Load => wgpu::LoadOp::Load,
        };

        let depth_load = match depth_load_op {
            WLoadOp::Clear => wgpu::LoadOp::Clear(depth_clear_value),
            WLoadOp::Load => wgpu::LoadOp::Load,
        };

        // Clone the views
        let color_view_inner = if color_view.is_surface_texture() {
            None
        } else {
            color_view.inner().cloned()
        };

        let depth_view_inner = depth_view.inner().cloned();

        log::info!(
            "beginRenderPassWithDepth: depth_view.inner().is_some()={}, depth_view_inner.is_some()={}",
            depth_view.inner().is_some(),
            depth_view_inner.is_some()
        );

        let config = RenderPassConfig {
            color_view: color_view_inner,
            depth_view: depth_view_inner,
            clear_color,
            color_load_op,
            depth_load_op: depth_load,
            depth_write: true,
        };

        let encoder_index = self.render_passes.len();

        WRenderPassEncoder {
            device_state: self.device_state.clone(),
            config,
            commands: Vec::new(),
            encoder_index,
        }
    }

    /// Finish the command encoder and retrieve all recorded passes
    pub fn finish(&mut self) -> WCommandBuffer {
        // Get all pending passes from thread-local storage
        let render_passes = take_pending_passes();
        log::debug!("Finishing command encoder with {} render passes", render_passes.len());

        let cmd_buf = WCommandBuffer {
            device_state: self.device_state.clone(),
            render_passes,
        };

        // Store for later submission by queue.submit()
        set_pending_command_buffer(cmd_buf);

        // Return a dummy - the real one is stored
        WCommandBuffer {
            device_state: self.device_state.clone(),
            render_passes: Vec::new(),
        }
    }
}

/// Command buffer (result of finishing a command encoder)
#[wasm_bindgen]
pub struct WCommandBuffer {
    device_state: Arc<RefCell<DeviceState>>,
    render_passes: Vec<(RenderPassConfig, Vec<RenderCommand>)>,
}

impl WCommandBuffer {
    /// Execute all recorded commands
    pub(crate) fn execute(&self) {
        let state = self.device_state.borrow();

        // Get surface texture for this frame
        let surface_texture = match state.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                return;
            }
        };

        let surface_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create encoder and execute all passes
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("main encoder"),
        });

        for (config, commands) in &self.render_passes {
            // Use surface view if no custom view provided
            let color_view = config.color_view.as_ref().unwrap_or(&surface_view);

            log::info!(
                "Executing render pass: has_color_view={}, has_depth_view={}, depth_write={}, commands={}",
                config.color_view.is_some(),
                config.depth_view.is_some(),
                config.depth_write,
                commands.len()
            );

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: config.color_load_op.clone(),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: config.depth_view.as_ref().map(|view| {
                        wgpu::RenderPassDepthStencilAttachment {
                            view,
                            depth_ops: Some(wgpu::Operations {
                                load: config.depth_load_op.clone(),
                                store: if config.depth_write {
                                    wgpu::StoreOp::Store
                                } else {
                                    wgpu::StoreOp::Discard
                                },
                            }),
                            stencil_ops: None,
                        }
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                // Execute all recorded commands
                for cmd in commands {
                    match cmd {
                        RenderCommand::SetPipeline(pipeline) => {
                            render_pass.set_pipeline(pipeline);
                        }
                        RenderCommand::SetBindGroup { index, bind_group } => {
                            render_pass.set_bind_group(*index, bind_group, &[]);
                        }
                        RenderCommand::SetVertexBuffer { slot, buffer, offset } => {
                            render_pass.set_vertex_buffer(*slot, buffer.slice(*offset..));
                        }
                        RenderCommand::SetIndexBuffer { buffer, format, offset } => {
                            render_pass.set_index_buffer(buffer.slice(*offset..), *format);
                        }
                        RenderCommand::Draw {
                            vertex_count,
                            instance_count,
                            first_vertex,
                            first_instance,
                        } => {
                            render_pass.draw(*first_vertex..(*first_vertex + *vertex_count), *first_instance..(*first_instance + *instance_count));
                        }
                        RenderCommand::DrawIndexed {
                            index_count,
                            instance_count,
                            first_index,
                            base_vertex,
                            first_instance,
                        } => {
                            render_pass.draw_indexed(*first_index..(*first_index + *index_count), *base_vertex, *first_instance..(*first_instance + *instance_count));
                        }
                        RenderCommand::SetViewport {
                            x,
                            y,
                            width,
                            height,
                            min_depth,
                            max_depth,
                        } => {
                            render_pass.set_viewport(*x, *y, *width, *height, *min_depth, *max_depth);
                        }
                        RenderCommand::SetScissorRect { x, y, width, height } => {
                            render_pass.set_scissor_rect(*x, *y, *width, *height);
                        }
                    }
                }
            }
        }

        // Submit the command buffer
        state.queue.submit(std::iter::once(encoder.finish()));

        // Present the surface
        surface_texture.present();

        log::debug!("Executed {} render passes and presented", self.render_passes.len());
    }
}

// Thread-local storage for completed render passes
// This allows end() to store commands that finish() can retrieve
thread_local! {
    static PENDING_PASSES: RefCell<Vec<(RenderPassConfig, Vec<RenderCommand>)>> = const { RefCell::new(Vec::new()) };
    // The pending command buffer waiting to be submitted
    static PENDING_COMMAND_BUFFER: RefCell<Option<WCommandBuffer>> = const { RefCell::new(None) };
}

fn add_pending_pass(config: RenderPassConfig, commands: Vec<RenderCommand>) {
    PENDING_PASSES.with(|passes| {
        passes.borrow_mut().push((config, commands));
    });
}

fn take_pending_passes() -> Vec<(RenderPassConfig, Vec<RenderCommand>)> {
    PENDING_PASSES.with(|passes| {
        std::mem::take(&mut *passes.borrow_mut())
    })
}

/// Store a command buffer for later submission
pub(crate) fn set_pending_command_buffer(cmd_buf: WCommandBuffer) {
    PENDING_COMMAND_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(cmd_buf);
    });
}

/// Take and execute the pending command buffer
pub(crate) fn execute_pending_command_buffer() {
    PENDING_COMMAND_BUFFER.with(|buf| {
        if let Some(cmd_buf) = buf.borrow_mut().take() {
            cmd_buf.execute();
        } else {
            log::warn!("No pending command buffer to submit");
        }
    });
}

#[wasm_bindgen]
impl WRenderPassEncoder {
    /// Set the render pipeline
    #[wasm_bindgen(js_name = setPipeline)]
    pub fn set_pipeline(&mut self, pipeline: &WRenderPipeline) {
        log::debug!("Recording: set pipeline");
        self.commands.push(RenderCommand::SetPipeline(pipeline.inner().clone()));
    }

    /// Set a vertex buffer
    #[wasm_bindgen(js_name = setVertexBuffer)]
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &WBuffer, offset: u32) {
        log::debug!("Recording: set vertex buffer at slot {}, offset {}", slot, offset);
        self.commands.push(RenderCommand::SetVertexBuffer {
            slot,
            buffer: buffer.inner().clone(),
            offset: offset as u64,
        });
    }

    /// Set the index buffer
    #[wasm_bindgen(js_name = setIndexBuffer)]
    pub fn set_index_buffer(&mut self, buffer: &WBuffer, format: u32, offset: u32) {
        log::debug!("Recording: set index buffer, format={}, offset={}", format, offset);
        let index_format = if format == 1 {
            wgpu::IndexFormat::Uint32
        } else {
            wgpu::IndexFormat::Uint16
        };
        self.commands.push(RenderCommand::SetIndexBuffer {
            buffer: buffer.inner().clone(),
            format: index_format,
            offset: offset as u64,
        });
    }

    /// Set a bind group
    #[wasm_bindgen(js_name = setBindGroup)]
    pub fn set_bind_group(&mut self, group_index: u32, bind_group: &WBindGroup) {
        log::debug!("Recording: set bind group at index {}", group_index);
        self.commands.push(RenderCommand::SetBindGroup {
            index: group_index,
            bind_group: bind_group.inner().clone(),
        });
    }

    /// Draw primitives
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        log::debug!(
            "Recording: draw vertices={}, instances={}, first_vertex={}, first_instance={}",
            vertex_count, instance_count, first_vertex, first_instance
        );
        self.commands.push(RenderCommand::Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        });
    }

    /// Draw indexed primitives
    #[wasm_bindgen(js_name = drawIndexed)]
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) {
        log::debug!(
            "Recording: draw indexed indices={}, instances={}, first_index={}, base_vertex={}, first_instance={}",
            index_count, instance_count, first_index, base_vertex, first_instance
        );
        self.commands.push(RenderCommand::DrawIndexed {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        });
    }

    /// Set viewport
    #[wasm_bindgen(js_name = setViewport)]
    pub fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        log::debug!(
            "Recording: set viewport ({}, {}) {}x{}, depth=[{}, {}]",
            x, y, width, height, min_depth, max_depth
        );
        self.commands.push(RenderCommand::SetViewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        });
    }

    /// Set scissor rect
    #[wasm_bindgen(js_name = setScissorRect)]
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        log::debug!("Recording: set scissor rect ({}, {}) {}x{}", x, y, width, height);
        self.commands.push(RenderCommand::SetScissorRect { x, y, width, height });
    }

    /// End the render pass
    pub fn end(self) {
        log::debug!("End render pass with {} commands", self.commands.len());
        // Store the completed pass in thread-local storage for finish() to retrieve
        add_pending_pass(self.config, self.commands);
    }
}
