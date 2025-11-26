/// <reference types="@webgpu/types" />

/**
 * Color value for clear operations
 */
export interface ClearColor {
  r: number;
  g: number;
  b: number;
  a: number;
}

/**
 * Frame context returned by beginFrame
 */
export interface FrameContext {
  commandEncoder: GPUCommandEncoder | import('../../pkg/wgpu_webgl_wasm.js').WCommandEncoder;
  renderPass: GPURenderPassEncoder | import('../../pkg/wgpu_webgl_wasm.js').WRenderPassEncoder;
}

/**
 * WebGPU context for native WebGPU backend
 */
export interface WebGPUContext {
  device: GPUDevice;
  queue: GPUQueue;
  context: GPUCanvasContext;
  format: GPUTextureFormat;
  canvas: HTMLCanvasElement;
}

/**
 * WebGL context for WASM-based WebGL2 backend
 */
export interface WebGLContext {
  device: import('../../pkg/wgpu_webgl_wasm.js').WDevice;
  queue: import('../../pkg/wgpu_webgl_wasm.js').WQueue;
  canvas: HTMLCanvasElement;
  wasm: typeof import('../../pkg/wgpu_webgl_wasm.js');
}

/**
 * Union type for either backend context
 */
export type GPUContext = WebGPUContext | WebGLContext;

/**
 * Union type for shader modules from either backend
 */
export type ShaderModule = GPUShaderModule | import('../../pkg/wgpu_webgl_wasm.js').WShaderModule;

/**
 * Union type for render pipelines from either backend
 */
export type RenderPipeline = GPURenderPipeline | import('../../pkg/wgpu_webgl_wasm.js').WRenderPipeline;

/**
 * Backend interface - common API for both WebGPU and WebGL backends
 */
export interface GPUBackend {
  init(canvas: HTMLCanvasElement): Promise<GPUContext>;
  createShaderModule(ctx: GPUContext, code: string, vertexEntry: string, fragmentEntry: string): ShaderModule;
  createRenderPipeline(ctx: GPUContext, shaderModule: ShaderModule, topology: string): RenderPipeline;
  beginFrame(ctx: GPUContext, clearColor: ClearColor): FrameContext;
  setPipeline(frame: FrameContext, pipeline: RenderPipeline): void;
  draw(frame: FrameContext, vertexCount: number, instanceCount: number, firstVertex: number, firstInstance: number): void;
  endFrame(ctx: GPUContext, frame: FrameContext): void;
}

/**
 * Type guard to check if context is WebGPU
 */
export function isWebGPUContext(ctx: GPUContext): ctx is WebGPUContext {
  return 'context' in ctx && 'format' in ctx;
}

/**
 * Type guard to check if context is WebGL
 */
export function isWebGLContext(ctx: GPUContext): ctx is WebGLContext {
  return 'wasm' in ctx;
}
