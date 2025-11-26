/// <reference types="@webgpu/types" />

import type { GPUBackend, WebGPUContext, ClearColor, FrameContext, ShaderModule, RenderPipeline, VertexBufferLayout, GPUBufferUnion, GPUContext } from './types.ts';

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

  createShaderModule(ctx: GPUContext, code: string, _vertexEntry: string, _fragmentEntry: string): GPUShaderModule {
    console.debug('WebGPU: createShaderModule');
    const webgpuCtx = ctx as WebGPUContext;
    return webgpuCtx.device.createShaderModule({ code });
  },

  createRenderPipeline(ctx: GPUContext, shaderModule: ShaderModule, _topology: string): GPURenderPipeline {
    console.debug('WebGPU: createRenderPipeline');
    const webgpuCtx = ctx as WebGPUContext;
    return webgpuCtx.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: shaderModule as GPUShaderModule,
        entryPoint: 'vs_main',
      },
      fragment: {
        module: shaderModule as GPUShaderModule,
        entryPoint: 'fs_main',
        targets: [{ format: webgpuCtx.format }],
      },
      primitive: { topology: 'triangle-list' },
    });
  },

  createRenderPipelineWithLayout(
    ctx: GPUContext,
    shaderModule: ShaderModule,
    _topology: string,
    vertexBufferLayout: VertexBufferLayout
  ): GPURenderPipeline {
    console.debug('WebGPU: createRenderPipelineWithLayout');
    const webgpuCtx = ctx as WebGPUContext;
    return webgpuCtx.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: shaderModule as GPUShaderModule,
        entryPoint: 'vs_main',
        buffers: [
          {
            arrayStride: vertexBufferLayout.arrayStride,
            attributes: vertexBufferLayout.attributes.map((attr) => ({
              shaderLocation: attr.shaderLocation,
              offset: attr.offset,
              format: attr.format,
            })),
          },
        ],
      },
      fragment: {
        module: shaderModule as GPUShaderModule,
        entryPoint: 'fs_main',
        targets: [{ format: webgpuCtx.format }],
      },
      primitive: { topology: 'triangle-list' },
    });
  },

  createBuffer(ctx: GPUContext, size: number, usage: number): GPUBuffer {
    console.debug('WebGPU: createBuffer');
    const webgpuCtx = ctx as WebGPUContext;
    return webgpuCtx.device.createBuffer({ size, usage });
  },

  createBufferWithData(ctx: GPUContext, data: ArrayBuffer | ArrayBufferView, usage: number): GPUBuffer {
    console.debug('WebGPU: createBufferWithData');
    const webgpuCtx = ctx as WebGPUContext;
    const buffer = webgpuCtx.device.createBuffer({
      size: data.byteLength,
      usage: usage | GPUBufferUsage.COPY_DST,
      mappedAtCreation: true,
    });
    const arrayBuffer = buffer.getMappedRange();
    if (data instanceof ArrayBuffer) {
      new Uint8Array(arrayBuffer).set(new Uint8Array(data));
    } else {
      new Uint8Array(arrayBuffer).set(new Uint8Array(data.buffer, data.byteOffset, data.byteLength));
    }
    buffer.unmap();
    return buffer;
  },

  writeBuffer(ctx: GPUContext, buffer: GPUBufferUnion, data: ArrayBuffer | ArrayBufferView, offset = 0): void {
    const webgpuCtx = ctx as WebGPUContext;
    if (data instanceof ArrayBuffer) {
      webgpuCtx.queue.writeBuffer(buffer as GPUBuffer, offset, data);
    } else {
      webgpuCtx.queue.writeBuffer(buffer as GPUBuffer, offset, data.buffer, data.byteOffset, data.byteLength);
    }
  },

  beginFrame(ctx: GPUContext, clearColor: ClearColor): FrameContext {
    const webgpuCtx = ctx as WebGPUContext;
    const commandEncoder = webgpuCtx.device.createCommandEncoder();
    const textureView = webgpuCtx.context.getCurrentTexture().createView();
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

  setVertexBuffer(frame: FrameContext, slot: number, buffer: GPUBufferUnion, offset = 0): void {
    (frame.renderPass as GPURenderPassEncoder).setVertexBuffer(slot, buffer as GPUBuffer, offset);
  },

  draw(frame: FrameContext, vertexCount: number, instanceCount: number, firstVertex: number, firstInstance: number): void {
    (frame.renderPass as GPURenderPassEncoder).draw(vertexCount, instanceCount, firstVertex, firstInstance);
  },

  endFrame(ctx: GPUContext, frame: FrameContext): void {
    const webgpuCtx = ctx as WebGPUContext;
    (frame.renderPass as GPURenderPassEncoder).end();
    webgpuCtx.queue.submit([(frame.commandEncoder as GPUCommandEncoder).finish()]);
  },
};
