//! WebGL2 backend that provides a WebGPU-like API
//!
//! This module exposes functions that mirror the WebGPU API but translate
//! to WebGL2 calls internally.

mod bind_group;
mod buffer;
mod command;
mod device;
mod pipeline;
mod sampler;
mod shader;
mod texture;
mod types;

pub use bind_group::*;
pub use buffer::*;
pub use command::*;
pub use device::*;
pub use pipeline::*;
pub use sampler::*;
pub use shader::*;
pub use texture::*;
pub use types::*;
