//! Sampler wrapper

use wasm_bindgen::prelude::*;
use std::sync::atomic::Ordering;
use super::device::WDevice;
use super::stats::SAMPLER_COUNT;

/// Address mode for texture sampling
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WAddressMode {
    ClampToEdge = 0,
    Repeat = 1,
    MirrorRepeat = 2,
}

impl WAddressMode {
    fn to_wgpu(self) -> wgpu::AddressMode {
        match self {
            Self::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            Self::Repeat => wgpu::AddressMode::Repeat,
            Self::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
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
    fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            Self::Nearest => wgpu::FilterMode::Nearest,
            Self::Linear => wgpu::FilterMode::Linear,
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

impl WMipmapFilterMode {
    fn to_wgpu(self) -> wgpu::MipmapFilterMode {
        match self {
            Self::Nearest => wgpu::MipmapFilterMode::Nearest,
            Self::Linear => wgpu::MipmapFilterMode::Linear,
        }
    }
}

/// Compare function for depth/shadow sampling
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
    pub(crate) fn to_wgpu(self) -> wgpu::CompareFunction {
        match self {
            Self::Never => wgpu::CompareFunction::Never,
            Self::Less => wgpu::CompareFunction::Less,
            Self::Equal => wgpu::CompareFunction::Equal,
            Self::LessEqual => wgpu::CompareFunction::LessEqual,
            Self::Greater => wgpu::CompareFunction::Greater,
            Self::NotEqual => wgpu::CompareFunction::NotEqual,
            Self::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            Self::Always => wgpu::CompareFunction::Always,
        }
    }
}

/// Compare function for depth sampling (includes None option)
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
    fn to_wgpu(self) -> Option<wgpu::CompareFunction> {
        match self {
            Self::None => None,
            Self::Never => Some(wgpu::CompareFunction::Never),
            Self::Less => Some(wgpu::CompareFunction::Less),
            Self::Equal => Some(wgpu::CompareFunction::Equal),
            Self::LessEqual => Some(wgpu::CompareFunction::LessEqual),
            Self::Greater => Some(wgpu::CompareFunction::Greater),
            Self::NotEqual => Some(wgpu::CompareFunction::NotEqual),
            Self::GreaterEqual => Some(wgpu::CompareFunction::GreaterEqual),
            Self::Always => Some(wgpu::CompareFunction::Always),
        }
    }
}

/// WebGPU Sampler wrapper
#[wasm_bindgen]
pub struct WSampler {
    pub(crate) inner: wgpu::Sampler,
}

impl WSampler {
    pub(crate) fn inner(&self) -> &wgpu::Sampler {
        &self.inner
    }
}

impl Drop for WSampler {
    fn drop(&mut self) {
        SAMPLER_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Create a sampler with full configuration
#[wasm_bindgen(js_name = createSampler)]
pub fn create_sampler(
    device: &WDevice,
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
    let state = device.state();
    let state = state.borrow();

    // max_anisotropy must be >= 1, clamp to valid range
    let anisotropy = max_anisotropy.max(1).min(16);

    let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: address_mode_u.to_wgpu(),
        address_mode_v: address_mode_v.to_wgpu(),
        address_mode_w: address_mode_w.to_wgpu(),
        mag_filter: mag_filter.to_wgpu(),
        min_filter: min_filter.to_wgpu(),
        mipmap_filter: mipmap_filter.to_wgpu(),
        lod_min_clamp,
        lod_max_clamp,
        compare: compare.to_wgpu(),
        anisotropy_clamp: anisotropy,
        ..Default::default()
    });

    log::debug!("Created sampler with lod=[{}, {}], compare={:?}, anisotropy={}",
        lod_min_clamp, lod_max_clamp, compare, anisotropy);

    SAMPLER_COUNT.fetch_add(1, Ordering::Relaxed);

    Ok(WSampler { inner: sampler })
}
