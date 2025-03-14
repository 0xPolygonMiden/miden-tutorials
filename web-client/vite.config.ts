import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    target: 'esnext',
    commonjsOptions: {
      include: [/node_modules/], // Ensures dependencies in node_modules are processed
      transformMixedEsModules: true, // Allows processing of mixed ESM & CJS modules
    },
    rollupOptions: {
      external: ['@demox-labs/miden-sdk'], // Mark the SDK as external to avoid Rollup transformation
      output: {
        format: "es", // Forces Rollup to emit ES modules
      },
    },
  },
  optimizeDeps: {
    include: ['@demox-labs/miden-sdk'], // Force Vite to pre-bundle the SDK using esbuild
    esbuildOptions: {
      target: "esnext",
      supported: {
        "top-level-await": true, // Ensure async/await is preserved
      },
    },
  },
});