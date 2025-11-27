# WGPU WebGL2 WASM Bridge Architecture

## Goal

Provide a WebGL2 fallback for the Polygon game engine when native WebGPU is unavailable.

## Design Decision: Why NOT Use wgpu-hal Directly

After analyzing wgpu-hal's GLES backend (`/wgpu/wgpu-hal/src/gles/`), we determined that
using it directly is **not the right approach** for this project because:

1. **wgpu-hal is designed for native use** - Its traits (`Device`, `Queue`, `CommandEncoder`)
   are not designed for JavaScript interop via wasm-bindgen

2. **Heavy dependencies** - wgpu-hal + wgpu-core add significant WASM binary size

3. **API mismatch** - Polygon expects `wgpu*` C-style function calls with heap indices
   and C struct pointers, not Rust trait methods

Instead, we use **only Naga** (the shader transpiler) from wgpu, and implement
a thin WebGL2 layer ourselves. This is what wgpu-hal does internally anyway.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Polygon (Jai WASM)                           │
│  Calls: wgpuDeviceCreateBuffer(deviceIndex, descriptorPtr)      │
└─────────────────────┬───────────────────────────────────────────┘
                      │ JavaScript import
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│              JavaScript Wrapper Layer (in Polygon)              │
│  src/javascript/WebGPU/FromJs/wgpuDeviceCreateBuffer.webgl.ts   │
│  - Reads C struct from Polygon's WASM memory                    │
│  - Calls wgpu-webgl-wasm's exported functions                   │
│  - Manages heap indices                                         │
└─────────────────────┬───────────────────────────────────────────┘
                      │ wasm-bindgen call
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│              wgpu-webgl-wasm (this repo)                        │
│  - Thin WebGL2 wrappers using glow                              │
│  - WGSL→GLSL transpilation via Naga                             │
│  - Exports: WDevice, WQueue, WBuffer, WShaderModule, etc.       │
└─────────────────────┬───────────────────────────────────────────┘
                      │ glow calls
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    WebGL2 Context                               │
└─────────────────────────────────────────────────────────────────┘
```

## Dependencies (Minimal)

```toml
[dependencies]
naga = { version = "27", features = ["wgsl-in", "glsl-out"] }  # Shader transpilation
glow = "0.16"                                                   # Type-safe WebGL2
wasm-bindgen = "0.2"                                           # JS interop
web-sys = "0.3"                                                # DOM/WebGL bindings
```

**NOT included**: wgpu-core, wgpu-hal (too heavy, wrong API shape)

## Key Components

### 1. Naga Shader Transpiler

Converts WGSL shaders to GLSL ES 300 (WebGL2 compatible):

```rust
use naga::front::wgsl;
use naga::back::glsl;

let module = wgsl::parse_str(wgsl_source)?;
let info = naga::valid::Validator::new(...).validate(&module)?;

let mut writer = glsl::Writer::new(&mut output, &module, &info, &options, ...)?;
writer.write()?;
```

### 2. Coordinate System Handling

WebGPU uses:
- Y-down in NDC (clip space) - for framebuffer
- Z in [0, 1] depth range

OpenGL/WebGL uses:
- Y-up in NDC
- Z in [-1, 1] depth range

Naga's `WriterFlags::ADJUST_COORDINATE_SPACE` handles Z remapping.
Y-flip is handled during framebuffer blit (like wgpu-hal does).

### 3. glow WebGL2 Bindings

Type-safe wrapper around WebGL2:

```rust
use glow::HasContext;

let gl = glow::Context::from_webgl2_context(webgl2_ctx);
unsafe {
    let buffer = gl.create_buffer()?;
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &data, glow::STATIC_DRAW);
}
```

## Memory Architecture

**Challenge**: Polygon WASM and wgpu-webgl-wasm have separate linear memories.

**Solution**: JavaScript mediates data transfer:

```typescript
// In Polygon's JavaScript layer
function wgpuQueueWriteBuffer(queueIndex, bufferIndex, offset, dataPtr, size) {
    // Read data from Polygon's WASM memory
    const data = new Uint8Array(polygonWasm.memory.buffer, Number(dataPtr), Number(size));

    // Get WebGL buffer from wgpu-webgl-wasm heap
    const buffer = wgpuWebgl.heapGet(bufferIndex);

    // Write to WebGL buffer
    wgpuWebgl.queue_write_buffer(queueIndex, bufferIndex, offset, data);
}
```

## File Structure

```
wgpu-webgl-wasm/
├── Cargo.toml              # Minimal dependencies
├── src/
│   ├── lib.rs              # WASM entry point
│   └── webgl_backend/
│       ├── mod.rs          # Module exports
│       ├── device.rs       # WDevice, WQueue
│       ├── shader.rs       # Naga WGSL→GLSL
│       ├── buffer.rs       # WBuffer
│       ├── pipeline.rs     # WRenderPipeline, WBindGroup
│       ├── command.rs      # WCommandEncoder, WRenderPass
│       └── types.rs        # Enums (topology, vertex format, etc.)
├── pkg/                    # Generated JS bindings
└── tests/                  # Shader transpilation tests
```

## API Surface

### Exported Types (wasm-bindgen)

| Type | WebGPU Equivalent | Purpose |
|------|-------------------|---------|
| WDevice | GPUDevice | Resource creation |
| WQueue | GPUQueue | Command submission |
| WBuffer | GPUBuffer | GPU memory |
| WShaderModule | GPUShaderModule | Compiled shaders |
| WRenderPipeline | GPURenderPipeline | Render state |
| WBindGroup | GPUBindGroup | Resource bindings |
| WCommandEncoder | GPUCommandEncoder | Command recording |
| WRenderPassEncoder | GPURenderPassEncoder | Render commands |

### Key Functions

```rust
// Device creation
pub fn create_device(canvas: &HtmlCanvasElement) -> Result<WDevice, JsValue>;

// Resource creation
impl WDevice {
    pub fn create_buffer(&self, size: u32, usage: u32) -> WBuffer;
    pub fn create_shader_module(&self, wgsl: &str, vs_entry: &str, fs_entry: &str) -> WShaderModule;
    pub fn create_render_pipeline(&self, shader: &WShaderModule, ...) -> WRenderPipeline;
}

// Command recording
impl WCommandEncoder {
    pub fn begin_render_pass(&self, ...) -> WRenderPassEncoder;
}

impl WRenderPassEncoder {
    pub fn set_pipeline(&self, pipeline: &WRenderPipeline);
    pub fn set_vertex_buffer(&self, slot: u32, buffer: &WBuffer, offset: u32);
    pub fn draw(&self, vertex_count: u32, instance_count: u32, ...);
    pub fn end(&self);
}

// Submission
impl WQueue {
    pub fn submit(&self);
    pub fn write_buffer(&self, buffer: &WBuffer, offset: u32, data: &[u8]);
}
```

## Integration with Polygon

### Step 1: Backend Detection (in Polygon)

```typescript
const useWebGL = !navigator.gpu;
const backend = useWebGL ? await loadWebGLBackend() : await loadWebGPUBackend();
```

### Step 2: Create WebGL-specific Wrapper Functions

For each `wgpu*` function Polygon calls, create a `.webgl.ts` variant:

```typescript
// wgpuDeviceCreateBuffer.webgl.ts
export function wgpuDeviceCreateBuffer(deviceHeapIndex: bigint, descriptorPtr: bigint) {
    // Read descriptor from Polygon's WASM memory
    const descriptor = WGPUBufferDescriptorFromBuffer(descriptorPtr);

    // Get device from wgpu-webgl-wasm
    const device = wgpuWebgl.heapGet(deviceHeapIndex);

    // Create buffer via wgpu-webgl-wasm
    const buffer = device.create_buffer(descriptor.size, descriptor.usage);

    // Store in heap, return index
    return wgpuWebgl.heapAdd(buffer);
}
```

### Step 3: Runtime Switching

```typescript
// WasmRuntimeWgpu.ts
const wgpuFunctions = useWebGL
    ? await import('./webgl/*.webgl.ts')
    : await import('./webgpu/*.ts');
```

## Testing

```bash
# Build WASM
npm run build:wasm

# Start dev server
npm run dev

# Open browser to test pages
# http://localhost:8080/triangle.html
# http://localhost:8080/triangle_buffer.html
```

## Status

- [x] Naga WGSL→GLSL transpilation
- [x] Basic device/queue creation
- [x] Buffer creation and data upload
- [x] Shader module compilation
- [x] Simple render pipeline
- [x] Draw calls (triangle demo)
- [ ] Bind groups (uniforms, textures)
- [ ] Index buffers
- [ ] Texture support
- [ ] Full Polygon integration
