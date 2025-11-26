import type { GPUBackend, WebGLContext, ClearColor, FrameContext, ShaderModule, RenderPipeline, GPUContext, VertexBufferLayout, GPUBufferUnion } from './types.ts';
import type * as WasmTypes from '../../pkg/wgpu_webgl_wasm.js';

/**
 * Type alias for the WASM module exports
 */
type WasmModule = typeof WasmTypes;

/**
 * Map GPUVertexFormat string to WVertexFormat enum
 */
function mapVertexFormat(wasm: WasmModule, format: GPUVertexFormat): WasmTypes.WVertexFormat {
  switch (format) {
    case 'float32': return wasm.WVertexFormat.Float32;
    case 'float32x2': return wasm.WVertexFormat.Float32x2;
    case 'float32x3': return wasm.WVertexFormat.Float32x3;
    case 'float32x4': return wasm.WVertexFormat.Float32x4;
    case 'uint32': return wasm.WVertexFormat.Uint32;
    case 'sint32': return wasm.WVertexFormat.Sint32;
    default:
      console.warn(`Unsupported vertex format: ${format}, defaulting to float32x4`);
      return wasm.WVertexFormat.Float32x4;
  }
}

/**
 * Buffer usage flags matching WebGPU
 */
const BufferUsage = {
  MAP_READ: 0x0001,
  MAP_WRITE: 0x0002,
  COPY_SRC: 0x0004,
  COPY_DST: 0x0008,
  INDEX: 0x0010,
  VERTEX: 0x0020,
  UNIFORM: 0x0040,
  STORAGE: 0x0080,
  INDIRECT: 0x0100,
  QUERY_RESOLVE: 0x0200,
};

/**
 * WebGL2 backend via WASM implementation
 */
export const WebGLBackend: GPUBackend = {
  async init(canvas: HTMLCanvasElement): Promise<WebGLContext> {
    console.info('Initializing WebGL backend via WASM...');

    // Load WASM module (dynamic import)
    const wasm: WasmModule = await import('../../pkg/wgpu_webgl_wasm.js');
    await wasm.default();
    console.info('WASM module loaded');

    // Create device (this gets the WebGL2 context internally)
    const device = wasm.createDevice(canvas);
    const queue = device.getQueue();

    return {
      device,
      queue,
      canvas,
      wasm,
    };
  },

  createShaderModule(ctx: GPUContext, code: string, vertexEntry: string, fragmentEntry: string): ShaderModule {
    console.debug('WebGL: createShaderModule (transpiling WGSL -> GLSL)');
    const webglCtx = ctx as WebGLContext;
    return webglCtx.wasm.createShaderModule(webglCtx.device, code, vertexEntry, fragmentEntry);
  },

  createRenderPipeline(ctx: GPUContext, shaderModule: ShaderModule, _topology: string): RenderPipeline {
    console.debug('WebGL: createRenderPipeline');
    const webglCtx = ctx as WebGLContext;
    return webglCtx.wasm.createRenderPipeline(
      webglCtx.device,
      shaderModule as WasmTypes.WShaderModule,
      webglCtx.wasm.WPrimitiveTopology.TriangleList
    );
  },

  createRenderPipelineWithLayout(
    ctx: GPUContext,
    shaderModule: ShaderModule,
    _topology: string,
    vertexBufferLayout: VertexBufferLayout
  ): RenderPipeline {
    console.debug('WebGL: createRenderPipelineWithLayout');
    const webglCtx = ctx as WebGLContext;

    // Create vertex buffer layout
    const layout = new webglCtx.wasm.WVertexBufferLayout(vertexBufferLayout.arrayStride);
    for (const attr of vertexBufferLayout.attributes) {
      layout.addAttribute(
        attr.shaderLocation,
        attr.offset,
        mapVertexFormat(webglCtx.wasm, attr.format)
      );
    }

    return webglCtx.wasm.createRenderPipelineWithLayout(
      webglCtx.device,
      shaderModule as WasmTypes.WShaderModule,
      webglCtx.wasm.WPrimitiveTopology.TriangleList,
      layout
    );
  },

  createBuffer(ctx: GPUContext, size: number, usage: number): GPUBufferUnion {
    console.debug('WebGL: createBuffer');
    const webglCtx = ctx as WebGLContext;
    return webglCtx.wasm.createBuffer(webglCtx.device, size, usage);
  },

  createBufferWithData(ctx: GPUContext, data: ArrayBuffer | ArrayBufferView, usage: number): GPUBufferUnion {
    console.debug('WebGL: createBufferWithData');
    const webglCtx = ctx as WebGLContext;
    let bytes: Uint8Array;
    if (data instanceof ArrayBuffer) {
      bytes = new Uint8Array(data);
    } else {
      bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    }
    return webglCtx.wasm.createBufferWithData(webglCtx.device, bytes, usage);
  },

  writeBuffer(_ctx: GPUContext, _buffer: GPUBufferUnion, _data: ArrayBuffer | ArrayBufferView, _offset = 0): void {
    console.debug('WebGL: writeBuffer');
    // For WebGL, we'd need to implement buffer sub-data update
    // For now, this is a no-op warning
    console.warn('WebGL: writeBuffer not fully implemented yet, use createBufferWithData instead');
  },

  beginFrame(ctx: GPUContext, clearColor: ClearColor): FrameContext {
    const webglCtx = ctx as WebGLContext;
    const commandEncoder = webglCtx.wasm.createCommandEncoder(webglCtx.device);
    const renderPass = commandEncoder.beginRenderPass(
      clearColor.r,
      clearColor.g,
      clearColor.b,
      clearColor.a,
      webglCtx.wasm.WLoadOp.Clear
    );
    return { commandEncoder, renderPass };
  },

  setPipeline(frame: FrameContext, pipeline: RenderPipeline): void {
    (frame.renderPass as WasmTypes.WRenderPassEncoder).setPipeline(pipeline as WasmTypes.WRenderPipeline);
  },

  setVertexBuffer(frame: FrameContext, slot: number, buffer: GPUBufferUnion, offset = 0): void {
    (frame.renderPass as WasmTypes.WRenderPassEncoder).setVertexBuffer(slot, buffer as WasmTypes.WBuffer, offset);
  },

  draw(frame: FrameContext, vertexCount: number, instanceCount: number, firstVertex: number, firstInstance: number): void {
    (frame.renderPass as WasmTypes.WRenderPassEncoder).draw(vertexCount, instanceCount, firstVertex, firstInstance);
  },

  endFrame(ctx: GPUContext, frame: FrameContext): void {
    const webglCtx = ctx as WebGLContext;
    (frame.renderPass as WasmTypes.WRenderPassEncoder).end();
    (frame.commandEncoder as WasmTypes.WCommandEncoder).finish();
    webglCtx.queue.submit();
  },
};

// Export buffer usage constants for convenience
export { BufferUsage };
