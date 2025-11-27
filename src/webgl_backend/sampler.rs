//! Sampler creation and management

use super::device::GlContextRef;
use glow::HasContext;
use wasm_bindgen::prelude::*;

/// Address mode for texture sampling
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WAddressMode {
    ClampToEdge = 0,
    Repeat = 1,
    MirrorRepeat = 2,
}

impl WAddressMode {
    pub fn to_gl(self) -> i32 {
        match self {
            WAddressMode::ClampToEdge => glow::CLAMP_TO_EDGE as i32,
            WAddressMode::Repeat => glow::REPEAT as i32,
            WAddressMode::MirrorRepeat => glow::MIRRORED_REPEAT as i32,
        }
    }
}

/// Filter mode for texture sampling
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WFilterMode {
    Nearest = 0,
    Linear = 1,
}

impl WFilterMode {
    pub fn to_gl_mag(self) -> i32 {
        match self {
            WFilterMode::Nearest => glow::NEAREST as i32,
            WFilterMode::Linear => glow::LINEAR as i32,
        }
    }

    pub fn to_gl_min(self, mipmap: WMipmapFilterMode) -> i32 {
        match (self, mipmap) {
            (WFilterMode::Nearest, WMipmapFilterMode::Nearest) => glow::NEAREST_MIPMAP_NEAREST as i32,
            (WFilterMode::Nearest, WMipmapFilterMode::Linear) => glow::NEAREST_MIPMAP_LINEAR as i32,
            (WFilterMode::Linear, WMipmapFilterMode::Nearest) => glow::LINEAR_MIPMAP_NEAREST as i32,
            (WFilterMode::Linear, WMipmapFilterMode::Linear) => glow::LINEAR_MIPMAP_LINEAR as i32,
        }
    }
}

/// Mipmap filter mode
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WMipmapFilterMode {
    Nearest = 0,
    Linear = 1,
}

/// Compare function for depth sampling
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WSamplerCompareFunction {
    /// No comparison (regular sampling)
    None = 0,
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}

impl WSamplerCompareFunction {
    pub fn to_gl(self) -> Option<i32> {
        match self {
            WSamplerCompareFunction::None => None,
            WSamplerCompareFunction::Never => Some(glow::NEVER as i32),
            WSamplerCompareFunction::Less => Some(glow::LESS as i32),
            WSamplerCompareFunction::Equal => Some(glow::EQUAL as i32),
            WSamplerCompareFunction::LessEqual => Some(glow::LEQUAL as i32),
            WSamplerCompareFunction::Greater => Some(glow::GREATER as i32),
            WSamplerCompareFunction::NotEqual => Some(glow::NOTEQUAL as i32),
            WSamplerCompareFunction::GreaterEqual => Some(glow::GEQUAL as i32),
            WSamplerCompareFunction::Always => Some(glow::ALWAYS as i32),
        }
    }
}

/// Sampler object - equivalent to GPUSampler
#[wasm_bindgen]
pub struct WSampler {
    context: GlContextRef,
    pub(crate) raw: glow::Sampler,
}

impl Drop for WSampler {
    fn drop(&mut self) {
        let ctx = self.context.borrow();
        unsafe {
            ctx.gl.delete_sampler(self.raw);
        }
        log::debug!("Sampler destroyed");
    }
}

/// Create a sampler with full configuration
#[wasm_bindgen(js_name = createSampler)]
pub fn create_sampler(
    device: &super::WDevice,
    address_mode_u: WAddressMode,
    address_mode_v: WAddressMode,
    address_mode_w: WAddressMode,
    mag_filter: WFilterMode,
    min_filter: WFilterMode,
    mipmap_filter: WMipmapFilterMode,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: WSamplerCompareFunction,
    max_anisotropy: u16,
) -> Result<WSampler, JsValue> {
    let context = device.context();
    let ctx = context.borrow();

    unsafe {
        let sampler = ctx
            .gl
            .create_sampler()
            .map_err(|e| JsValue::from_str(&format!("Failed to create sampler: {}", e)))?;

        // Set wrap modes
        ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_S, address_mode_u.to_gl());
        ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_T, address_mode_v.to_gl());
        ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_WRAP_R, address_mode_w.to_gl());

        // Set filter modes
        ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_MAG_FILTER, mag_filter.to_gl_mag());
        ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_MIN_FILTER, min_filter.to_gl_min(mipmap_filter));

        // Set LOD clamps
        ctx.gl.sampler_parameter_f32(sampler, glow::TEXTURE_MIN_LOD, lod_min_clamp);
        ctx.gl.sampler_parameter_f32(sampler, glow::TEXTURE_MAX_LOD, lod_max_clamp);

        // Set compare function if specified
        if let Some(compare_func) = compare.to_gl() {
            ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_COMPARE_MODE, glow::COMPARE_REF_TO_TEXTURE as i32);
            ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_COMPARE_FUNC, compare_func);
        } else {
            ctx.gl.sampler_parameter_i32(sampler, glow::TEXTURE_COMPARE_MODE, glow::NONE as i32);
        }

        // Set anisotropy if supported and requested
        if max_anisotropy > 1 {
            // EXT_texture_filter_anisotropic
            // Note: WebGL2 may not support this without extension
            ctx.gl.sampler_parameter_f32(sampler, glow::TEXTURE_MAX_ANISOTROPY_EXT, max_anisotropy as f32);
        }

        log::info!(
            "Sampler created: wrap=({:?},{:?},{:?}), filter=({:?},{:?}), mipmap={:?}",
            address_mode_u, address_mode_v, address_mode_w,
            mag_filter, min_filter, mipmap_filter
        );

        Ok(WSampler {
            context: context.clone(),
            raw: sampler,
        })
    }
}
