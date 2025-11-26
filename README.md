# wgpu-webgl-wasm

A WASM module that provides WebGPU-like functionality via WebGL2, using wgpu's GLES backend and Naga for WGSL→GLSL transpilation.

## Prerequisites

- Rust (rustup)
- wasm32-unknown-unknown target
- wasm-bindgen-cli
- Node.js (for Vite dev server)

### Install Prerequisites

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli

# Install Node.js dependencies
npm install
```

## Development

### Quick Start

```bash
# Build WASM module
npm run build:wasm

# Start Vite dev server
npm run dev
```

Then open http://localhost:8080/triangle.html in your browser.

### Available Scripts

| Script | Description |
|--------|-------------|
| `npm run dev` | Start Vite dev server with HMR |
| `npm run build` | Build for production |
| `npm run preview` | Preview production build |
| `npm run typecheck` | Run TypeScript type checking |
| `npm run build:wasm` | Build WASM (debug) |
| `npm run build:wasm:release` | Build WASM (release, optimized) |

### Building WASM Manually

Building requires **two steps**:
1. `cargo build` - compiles Rust to WASM
2. `wasm-bindgen` - generates JavaScript bindings

```bash
# Development build
cargo build --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/debug/wgpu_webgl_wasm.wasm

# Release build
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/wgpu_webgl_wasm.wasm
```

## Project Structure

```
wgpu-webgl-wasm/
├── Cargo.toml              # Rust dependencies
├── package.json            # Node.js dependencies & scripts
├── tsconfig.json           # TypeScript configuration
├── vite.config.ts          # Vite configuration
├── src/
│   ├── lib.rs              # Rust entry point
│   ├── webgl_backend/      # Rust WebGL2 backend
│   │   ├── mod.rs
│   │   ├── device.rs       # WDevice, WQueue
│   │   ├── shader.rs       # WShaderModule, WGSL→GLSL
│   │   ├── pipeline.rs     # WRenderPipeline
│   │   ├── buffer.rs       # WBuffer
│   │   ├── command.rs      # WCommandEncoder, WRenderPassEncoder
│   │   └── types.rs        # Enums and constants
│   └── ts/                 # TypeScript source
│       ├── types.ts        # Type definitions
│       ├── webgpu-backend.ts   # Native WebGPU backend
│       ├── webgl-backend.ts    # WASM WebGL2 backend
│       ├── logger.ts       # Logging utilities
│       └── triangle.ts     # Triangle demo entry point
├── pkg/                    # Generated WASM + JS bindings
├── triangle.html           # Triangle demo page
└── test.html               # Basic tests page
```

## API Usage

```typescript
import { WebGPUBackend } from './src/ts/webgpu-backend.js';
import { WebGLBackend } from './src/ts/webgl-backend.js';
import type { GPUBackend, GPUContext, ClearColor } from './src/ts/types.js';

// Choose backend based on WebGPU support
const backend: GPUBackend = navigator.gpu ? WebGPUBackend : WebGLBackend;

// Initialize
const ctx: GPUContext = await backend.init(canvas);

// Create shader module (WGSL - transpiled to GLSL for WebGL)
const shaderModule = backend.createShaderModule(ctx, wgslCode, 'vs_main', 'fs_main');

// Create render pipeline
const pipeline = backend.createRenderPipeline(ctx, shaderModule, 'triangle-list');

// Render frame
const clearColor: ClearColor = { r: 0.1, g: 0.1, b: 0.15, a: 1.0 };
const frame = backend.beginFrame(ctx, clearColor);
backend.setPipeline(frame, pipeline);
backend.draw(frame, 3, 1, 0, 0);
backend.endFrame(ctx, frame);
```

## How It Works

This module uses:
- **naga**: Shader transpiler (WGSL → GLSL ES 300)
- **glow**: Type-safe WebGL2 bindings
- **wasm-bindgen**: Rust ↔ JavaScript interop
- **Vite**: Modern build tool with TypeScript support

When compiled for `wasm32-unknown-unknown`, the module:
1. Receives WGSL shader code from JavaScript
2. Transpiles WGSL to GLSL ES 300 using Naga
3. Compiles GLSL shaders via WebGL2
4. Executes draw calls via WebGL2

The TypeScript abstraction layer (`GPUBackend`) provides the same API for both native WebGPU and the WASM-based WebGL2 fallback.
