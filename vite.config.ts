import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: Number(process.env.E2E_WEB_PORT || 5173),
    allowedHosts: true,
    proxy: {
      '/api': {
        target: `http://127.0.0.1:${process.env.API_PORT || 3001}`,
        changeOrigin: false,
        xfwd: true,
      },
    },
  },
  build: {
    outDir: 'dist-web',
    rollupOptions: {
      output: { manualChunks: { three: ['three'], react: ['react', 'react-dom'] } },
    },
  },
});
