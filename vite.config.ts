import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    rollupOptions: {
      input: {
        triangle: resolve(__dirname, 'triangle.html'),
        test: resolve(__dirname, 'test.html'),
      },
    },
  },
  server: {
    port: 8080,
  },
});
