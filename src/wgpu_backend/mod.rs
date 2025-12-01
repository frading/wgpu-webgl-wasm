//! wgpu-based WebGL backend
//!
//! This module wraps wgpu types and exposes them via wasm-bindgen.
//! The goal is to maintain API compatibility with the previous
//! hand-rolled WebGL implementation.

mod device;
mod buffer;
mod shader;
mod texture;
mod sampler;
mod pipeline;
mod bind_group;
mod command;
mod types;
mod stats;

pub use device::*;
pub use buffer::*;
pub use shader::*;
pub use texture::*;
pub use sampler::*;
pub use pipeline::*;
pub use bind_group::*;
pub use command::*;
pub use types::*;
pub use stats::*;
