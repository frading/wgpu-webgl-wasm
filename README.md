# wgpu-webgl-wasm

A WASM module that provides WebGPU-like functionality via WebGL2, using wgpu's GLES backend.

## Prerequisites

- Rust (rustup)
- wasm32-unknown-unknown target
- wasm-bindgen-cli

### Install Prerequisites

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli
```

## Building

### Development Build

```bash
# Compile to WASM
cargo build --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/debug/wgpu_webgl_wasm.wasm
```

### Release Build

```bash
# Compile to WASM (optimized)
cargo build --target wasm32-unknown-unknown --release

# Generate JS bindings
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/wgpu_webgl_wasm.wasm
```

## Testing

Start a local HTTP server (required for WASM):

```bash
python3 -m http.server 8080
```

Open http://localhost:8080/test.html in your browser.

## Usage

```javascript
import init, {
    test_wasm,
    get_webgl2_context,
    transpile_wgsl_to_glsl
} from './pkg/wgpu_webgl_wasm.js';

// Initialize the WASM module
await init();

// Test basic functionality
console.log(test_wasm());

// Get WebGL2 context from a canvas
const canvas = document.getElementById('myCanvas');
const gl = get_webgl2_context(canvas);

// Transpile WGSL shader to GLSL ES 300
const wgsl = `
@vertex
fn main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
`;
const glsl = transpile_wgsl_to_glsl(wgsl);
console.log(glsl);
```

## Project Structure

```
wgpu-webgl-wasm/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # WASM exports
├── pkg/                # Generated (after build)
│   ├── wgpu_webgl_wasm.js
│   ├── wgpu_webgl_wasm.d.ts
│   └── wgpu_webgl_wasm_bg.wasm
├── test.html           # Test page
├── README.md           # This file
└── ARCHITECTURE.md     # Design documentation
```

## How It Works

This module uses:
- **wgpu-core**: Safe Rust wrapper around wgpu-hal
- **wgpu-hal**: Hardware abstraction layer with GLES backend
- **naga**: Shader transpiler (WGSL → GLSL ES 300)
- **glow**: Type-safe WebGL bindings
- **wasm-bindgen**: Rust ↔ JavaScript interop

When compiled for `wasm32-unknown-unknown` with the `gles` feature, wgpu-hal automatically uses its WebGL2 backend (`wgpu-hal/src/gles/web.rs`).
