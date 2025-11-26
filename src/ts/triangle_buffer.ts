/// <reference types="@webgpu/types" />

import { WebGPUBackend } from './webgpu-backend.ts';
import { WebGLBackend, BufferUsage } from './webgl-backend.ts';
import { configureLogger, info, warn, error } from './logger.ts';
import type { GPUBackend, GPUContext, ClearColor, RenderPipeline, ShaderModule, GPUBufferUnion, VertexBufferLayout } from './types.ts';

// Shader source (WGSL) - reads position from vertex buffer at @location(0)
const SHADER_CODE = `
struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    return output;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.2, 0.8, 0.4, 1.0);  // Green triangle
}
`;

// Triangle vertex data (3 vertices, 2 floats each = 24 bytes)
const TRIANGLE_VERTICES = new Float32Array([
   0.0,  0.5,   // top
  -0.5, -0.5,   // bottom-left
   0.5, -0.5,   // bottom-right
]);

/**
 * Main entry point for the triangle buffer demo
 */
async function main(): Promise<void> {
  const canvas = document.getElementById('canvas') as HTMLCanvasElement | null;
  const status = document.getElementById('status') as HTMLElement | null;
  const logs = document.getElementById('logs') as HTMLElement | null;

  if (!canvas) {
    throw new Error('Canvas element not found');
  }

  // Configure logger to also output to the DOM
  configureLogger({ container: logs });

  let ctx: GPUContext | null = null;
  let backend: GPUBackend | null = null;

  // Try WebGPU first
  if (navigator.gpu) {
    try {
      backend = WebGPUBackend;
      ctx = await backend.init(canvas);
      if (status) {
        status.innerHTML = '<span class="webgpu">Using WebGPU (native)</span>';
      }
      info('WebGPU backend initialized');
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      warn(`WebGPU init failed: ${message}`);
      backend = null;
    }
  }

  // Fall back to WebGL
  if (!backend) {
    try {
      backend = WebGLBackend;
      ctx = await backend.init(canvas);
      if (status) {
        status.innerHTML = '<span class="webgl">Using WebGL2 (via wgpu WASM)</span>';
      }
      info('WebGL backend initialized');
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      if (status) {
        status.innerHTML = `<span class="error">Failed: ${message}</span>`;
      }
      error(`Both backends failed: ${message}`);
      throw e;
    }
  }

  if (!ctx || !backend) {
    throw new Error('Failed to initialize any graphics backend');
  }

  // ========================================
  // This code is THE SAME for both backends!
  // ========================================

  // Create vertex buffer with triangle data
  const vertexBuffer: GPUBufferUnion = backend.createBufferWithData(
    ctx,
    TRIANGLE_VERTICES,
    BufferUsage.VERTEX | BufferUsage.COPY_DST
  );
  info(`Vertex buffer created (${TRIANGLE_VERTICES.byteLength} bytes)`);

  // Define vertex buffer layout
  const vertexBufferLayout: VertexBufferLayout = {
    arrayStride: 2 * 4, // 2 floats * 4 bytes each = 8 bytes per vertex
    attributes: [
      {
        shaderLocation: 0,
        offset: 0,
        format: 'float32x2',
      },
    ],
  };

  // Create shader module (WGSL code - transpiled to GLSL for WebGL)
  const shaderModule: ShaderModule = backend.createShaderModule(ctx, SHADER_CODE, 'vs_main', 'fs_main');
  info('Shader module created');

  // Create render pipeline with vertex buffer layout
  const pipeline: RenderPipeline = backend.createRenderPipelineWithLayout(
    ctx,
    shaderModule,
    'triangle-list',
    vertexBufferLayout
  );
  info('Render pipeline with vertex layout created');

  // Clear color
  const clearColor: ClearColor = { r: 0.1, g: 0.1, b: 0.15, a: 1.0 };

  // Render loop
  let frameCount = 0;

  function render(): void {
    if (!ctx || !backend) return;

    // Begin frame
    const frame = backend.beginFrame(ctx, clearColor);

    // Set pipeline
    backend.setPipeline(frame, pipeline);

    // Bind vertex buffer at slot 0
    backend.setVertexBuffer(frame, 0, vertexBuffer);

    // Draw 3 vertices (1 triangle)
    backend.draw(frame, 3, 1, 0, 0);

    // End frame
    backend.endFrame(ctx, frame);

    frameCount++;
    if (frameCount === 1) {
      info('First frame rendered!');
    }

    // Uncomment for continuous rendering:
    // requestAnimationFrame(render);
  }

  render();
}

// Run main and handle errors
main().catch((e) => {
  const message = e instanceof Error ? e.message : String(e);
  error(`Unhandled error: ${message}`);
  console.error(e);
});
