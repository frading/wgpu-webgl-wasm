//! WebGL2 backend that provides a WebGPU-like API
//!
//! This module exposes functions that mirror the WebGPU API but translate
//! to WebGL2 calls internally.

mod device;
mod shader;
mod pipeline;
mod buffer;
mod command;
mod types;

pub use device::*;
pub use shader::*;
pub use pipeline::*;
pub use buffer::*;
pub use command::*;
pub use types::*;
