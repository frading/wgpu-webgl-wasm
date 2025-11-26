import type { GPUBackend, WebGLContext, ClearColor, FrameContext, ShaderModule, RenderPipeline, GPUContext } from './types.ts';
import type * as WasmTypes from '../../pkg/wgpu_webgl_wasm.js';

/**
 * Type alias for the WASM module exports
 */
type WasmModule = typeof WasmTypes;

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
