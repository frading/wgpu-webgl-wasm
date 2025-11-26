/// <reference types="@webgpu/types" />

import type { GPUBackend, WebGPUContext, ClearColor, FrameContext, ShaderModule, RenderPipeline } from './types.ts';

/**
 * Native WebGPU backend implementation
 */
export const WebGPUBackend: GPUBackend = {
  async init(canvas: HTMLCanvasElement): Promise<WebGPUContext> {
    console.info('Initializing WebGPU backend...');

    if (!navigator.gpu) {
      throw new Error('WebGPU not supported');
    }

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      throw new Error('No WebGPU adapter found');
    }
    console.info('WebGPU adapter obtained');

    const device = await adapter.requestDevice();
    console.info('WebGPU device created');

    const context = canvas.getContext('webgpu');
    if (!context) {
      throw new Error('Failed to get WebGPU context');
    }

    const format = navigator.gpu.getPreferredCanvasFormat();

    context.configure({
      device,
      format,
      alphaMode: 'premultiplied',
    });
    console.info(`Canvas configured with format: ${format}`);

    return {
      device,
      queue: device.queue,
      context,
      format,
      canvas,
    };
  },

  createShaderModule(ctx: WebGPUContext, code: string, _vertexEntry: string, _fragmentEntry: string): GPUShaderModule {
    console.debug('WebGPU: createShaderModule');
    return ctx.device.createShaderModule({ code });
  },

  createRenderPipeline(ctx: WebGPUContext, shaderModule: ShaderModule, _topology: string): GPURenderPipeline {
    console.debug('WebGPU: createRenderPipeline');
    return ctx.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: shaderModule as GPUShaderModule,
        entryPoint: 'vs_main',
      },
      fragment: {
        module: shaderModule as GPUShaderModule,
        entryPoint: 'fs_main',
        targets: [{ format: ctx.format }],
      },
      primitive: { topology: 'triangle-list' },
    });
  },

  beginFrame(ctx: WebGPUContext, clearColor: ClearColor): FrameContext {
    const commandEncoder = ctx.device.createCommandEncoder();
    const textureView = ctx.context.getCurrentTexture().createView();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: textureView,
          clearValue: clearColor,
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
    return { commandEncoder, renderPass };
  },

  setPipeline(frame: FrameContext, pipeline: RenderPipeline): void {
    (frame.renderPass as GPURenderPassEncoder).setPipeline(pipeline as GPURenderPipeline);
  },

  draw(frame: FrameContext, vertexCount: number, instanceCount: number, firstVertex: number, firstInstance: number): void {
    (frame.renderPass as GPURenderPassEncoder).draw(vertexCount, instanceCount, firstVertex, firstInstance);
  },

  endFrame(ctx: WebGPUContext, frame: FrameContext): void {
    (frame.renderPass as GPURenderPassEncoder).end();
    ctx.queue.submit([(frame.commandEncoder as GPUCommandEncoder).finish()]);
  },
};
