/// <reference types="@webgpu/types" />

import { WebGPUBackend } from './webgpu-backend.ts';
import { WebGLBackend } from './webgl-backend.ts';
import { configureLogger, info, warn, error } from './logger.ts';
import type { GPUBackend, GPUContext, ClearColor, RenderPipeline, ShaderModule } from './types.ts';

// Shader source (WGSL) - THE SAME shader is used for both paths!
const SHADER_CODE = `
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5)
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.5, 0.2, 1.0);
}
`;

/**
 * Main entry point for the triangle demo
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

  // Create shader module (WGSL code - transpiled to GLSL for WebGL)
  const shaderModule: ShaderModule = backend.createShaderModule(ctx, SHADER_CODE, 'vs_main', 'fs_main');
  info('Shader module created');

  // Create render pipeline
  const pipeline: RenderPipeline = backend.createRenderPipeline(ctx, shaderModule, 'triangle-list');
  info('Render pipeline created');

  // Clear color
  const clearColor: ClearColor = { r: 0.1, g: 0.1, b: 0.15, a: 1.0 };

  // Render loop
  let frameCount = 0;

  function render(): void {
    if (!ctx || !backend) return;

    // Begin frame
    const frame = backend.beginFrame(ctx, clearColor);

    // Set pipeline and draw
    backend.setPipeline(frame, pipeline);
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
