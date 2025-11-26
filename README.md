# wgpu-webgl-wasm

A WASM module that provides WebGPU-like functionality via WebGL2, using wgpu's GLES backend and Naga for WGSL→GLSL transpilation.

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

Building requires **two steps**:
1. `cargo build` - compiles Rust to WASM
2. `wasm-bindgen` - generates JavaScript bindings and optimizes the WASM

### Development Build

```bash
# Step 1: Compile Rust to WASM
cargo build --target wasm32-unknown-unknown

# Step 2: Generate JS bindings (REQUIRED after every cargo build!)
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/debug/wgpu_webgl_wasm.wasm
```

### Release Build

```bash
# Step 1: Compile Rust to WASM (optimized)
cargo build --target wasm32-unknown-unknown --release

# Step 2: Generate JS bindings
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/wgpu_webgl_wasm.wasm
```

### Quick Build (one-liner)

```bash
# Development
cargo build --target wasm32-unknown-unknown && wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/debug/wgpu_webgl_wasm.wasm

# Release
cargo build --target wasm32-unknown-unknown --release && wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/wgpu_webgl_wasm.wasm
```

## Testing

Start a local HTTP server (required for WASM):

```bash
python3 -m http.server 8080
```

Open in your browser:
- http://localhost:8080/triangle.html - Triangle rendering demo (WebGPU with WebGL2 fallback)
- http://localhost:8080/test.html - Basic WASM and shader transpilation tests

## API Overview

The module exposes a WebGPU-like API that translates to WebGL2 calls:

```javascript
import init, {
    createDevice,
    createShaderModule,
    createRenderPipeline,
    createCommandEncoder,
    WPrimitiveTopology,
    WLoadOp
} from './pkg/wgpu_webgl_wasm.js';

// Initialize WASM module
await init();

// Create device from canvas
const device = createDevice(canvas);
const queue = device.getQueue();

// Create shader module (WGSL is transpiled to GLSL internally)
const shaderModule = createShaderModule(device, wgslCode, 'vs_main', 'fs_main');

// Create render pipeline
const pipeline = createRenderPipeline(device, shaderModule, WPrimitiveTopology.TriangleList);

// Render frame
const encoder = createCommandEncoder(device);
const pass = encoder.beginRenderPass(0.1, 0.1, 0.15, 1.0, WLoadOp.Clear);
pass.setPipeline(pipeline);
pass.draw(3, 1, 0, 0);
pass.end();
encoder.finish();
queue.submit();
```

## Project Structure

```
wgpu-webgl-wasm/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── lib.rs              # Main entry point
│   └── webgl_backend/      # WebGL2 backend implementation
│       ├── mod.rs
│       ├── device.rs       # WDevice, WQueue
│       ├── shader.rs       # WShaderModule, WGSL→GLSL transpilation
│       ├── pipeline.rs     # WRenderPipeline
│       ├── buffer.rs       # WBuffer
│       ├── command.rs      # WCommandEncoder, WRenderPassEncoder
│       └── types.rs        # Enums and constants
├── pkg/                    # Generated (after wasm-bindgen)
│   ├── wgpu_webgl_wasm.js
│   ├── wgpu_webgl_wasm.d.ts
│   └── wgpu_webgl_wasm_bg.wasm
├── triangle.html           # Triangle demo with WebGPU/WebGL2 fallback
├── test.html               # Basic tests
├── README.md               # This file
└── ARCHITECTURE.md         # Design documentation
```

## How It Works

This module uses:
- **naga**: Shader transpiler (WGSL → GLSL ES 300)
- **glow**: Type-safe WebGL2 bindings
- **wasm-bindgen**: Rust ↔ JavaScript interop

When compiled for `wasm32-unknown-unknown`, the module:
1. Receives WGSL shader code from JavaScript
2. Transpiles WGSL to GLSL ES 300 using Naga
3. Compiles GLSL shaders via WebGL2
4. Executes draw calls via WebGL2

The JavaScript application uses the same rendering logic for both WebGPU and WebGL2 - only the backend implementation differs.
