import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    target: 'esnext', // Allow modern features like top-level await
  },
  optimizeDeps: {
    exclude: ['@demox-labs/miden-sdk'], // Exclude the SDK from optimization
  },
});
