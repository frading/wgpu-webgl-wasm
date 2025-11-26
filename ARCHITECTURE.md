# WGPU WebGL2 WASM Bridge Architecture

## Overview

This proof-of-concept demonstrates how to use wgpu's GLES backend (which automatically becomes WebGL2 when compiled for wasm32) to provide WebGPU-like functionality on browsers that don't support WebGPU natively.

## The Problem

Your current architecture:
```
Jai Code (wasm64) -> wgpuXxx() calls -> TypeScript shims -> browser WebGPU API
```

This only works on browsers with WebGPU support. For WebGL-only browsers, you need a translation layer.

## The Solution

For WebGL fallback:
```
Jai Code (wasm64) -> wgpuXxx() calls -> Modified TS shims -> wgpu-webgl-wasm (wasm32) -> WebGL2 API
```

## Key Components

### 1. Naga Shader Transpiler
- Converts WGSL shaders to GLSL ES 300 (WebGL2 compatible)
- Already integrated in wgpu-core
- Demonstrated in `transpile_wgsl_to_glsl()` function

### 2. wgpu-hal GLES Backend
- Complete WebGL2 implementation in `/wgpu-hal/src/gles/web.rs`
- Uses `glow` crate for type-safe WebGL bindings
- Handles all the WebGPU -> WebGL2 translation

### 3. wgpu-core
- High-level safe Rust API
- Resource management (buffers, textures, pipelines, etc.)
- Command encoding and submission

## Memory Architecture Challenge

**Critical Issue**: Two separate WASM modules cannot share linear memory by default.

Your Jai code compiles to wasm64, while wgpu-webgl-wasm compiles to wasm32. They have separate memory spaces.

### Solution Options

#### Option A: JavaScript Mediated Data Copy (Recommended for PoC)
```
Jai WASM Memory                    wgpu-webgl WASM Memory
     |                                     |
     v                                     v
[buffer data] --> JS copies --> [buffer data copy]
```

How it works:
1. Jai calls `wgpuDeviceCreateBuffer(descriptor_ptr)`
2. Your TS shim reads the descriptor from Jai's WASM memory
3. TS shim serializes/converts it for wgpu-webgl-wasm
4. TS shim calls `wgpu_create_buffer(...)` on wgpu-webgl-wasm
5. Returns handle back to Jai

Pros:
- Simple to implement
- Clear separation of concerns
- Works with existing TS shim architecture

Cons:
- Data copying overhead
- Potential for large memory usage with big buffers

#### Option B: Single WASM Memory (Complex)
Use WASM shared memory or link both modules into a single WASM.

Pros:
- Zero-copy data sharing
- Better performance

Cons:
- Requires wasm64 Rust target (experimental)
- Complex build setup
- May not be practical

#### Option C: Handle-Based API with Lazy Copies
Only copy data when actually needed (e.g., at draw time).

```javascript
// In your modified TS shim for WebGL:
function wgpuDeviceCreateBuffer(descriptor_ptr) {
    const descriptor = readDescriptorFromJaiMemory(descriptor_ptr);

    // Create a "lazy" buffer that tracks the Jai memory location
    const handle = wgpuWebgl.create_buffer_lazy({
        size: descriptor.size,
        usage: descriptor.usage,
        jai_memory_ptr: descriptor.mappedAtCreation ? getDataPtr() : null
    });

    return handle;
}

// Copy happens just before rendering
function wgpuQueueSubmit(...) {
    // Sync any dirty buffers from Jai memory to WebGL
    syncPendingBuffers();
    wgpuWebgl.queue_submit(...);
}
```

## Implementation Roadmap

### Phase 1: Shader Transpilation (DONE)
- [x] Naga WGSL -> GLSL ES 300 working
- [x] Basic WASM module structure

### Phase 2: Expand wgpu-webgl-wasm API
Add wasm-bindgen exports for:
- [ ] `create_instance()`
- [ ] `create_adapter(canvas)`
- [ ] `create_device(adapter)`
- [ ] `create_buffer(device, descriptor)`
- [ ] `create_texture(device, descriptor)`
- [ ] `create_shader_module(device, wgsl_code)`
- [ ] `create_render_pipeline(device, descriptor)`
- [ ] `create_bind_group(device, descriptor)`
- [ ] `begin_render_pass(encoder, descriptor)`
- [ ] `draw(pass, vertex_count, instance_count, ...)`
- [ ] `queue_submit(queue, command_buffers)`

### Phase 3: TS Shim Adaptation
Create WebGL-specific versions of your TypeScript shims:
```
src/javascript/WebGPU/FromJs/
├── wgpuDeviceCreateBindGroup.ts          # Current WebGPU version
├── wgpuDeviceCreateBindGroup.webgl.ts    # New WebGL version
```

### Phase 4: Runtime Detection & Switching
```typescript
// In your main entry point
const useWebGL = !navigator.gpu;

if (useWebGL) {
    await initWgpuWebglWasm();
    // Load WebGL shims
} else {
    // Use native WebGPU shims
}
```

## File Structure

```
wgpu-webgl-wasm/
├── Cargo.toml              # Rust dependencies
├── src/
│   └── lib.rs              # WASM exports
├── pkg/                    # Generated JS bindings
│   ├── wgpu_webgl_wasm.js
│   ├── wgpu_webgl_wasm.d.ts
│   └── wgpu_webgl_wasm_bg.wasm
├── test.html               # Test page
└── ARCHITECTURE.md         # This file
```

## Testing

Start a local server:
```bash
cd wgpu-webgl-wasm
python3 -m http.server 8080
```

Open http://localhost:8080/test.html

The test page demonstrates:
1. Basic WASM module loading
2. WebGL2 context creation from Rust/WASM
3. WGSL to GLSL shader transpilation

## Next Steps

1. **Expand the Rust API** to expose more wgpu functionality via wasm-bindgen
2. **Create a simple triangle example** that renders through the full wgpu -> WebGL2 path
3. **Design the TS shim API** to match your existing wgpuXxx patterns
4. **Implement data marshalling** between Jai WASM and wgpu-webgl WASM
