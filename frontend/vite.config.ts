import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
  server: {
    port: 5173,
    host: '0.0.0.0',
    // Hot-Reload Konfiguration für Docker
    hmr: {
      host: 'localhost',
      port: 5173,
      protocol: 'ws',
    },
    // Kein Proxy nötig - API läuft auf dem Host und wird direkt angesprochen
    // Der Frontend setzt VITE_API_URL oder nutzt das Fallback
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
  },
  resolve: {
    alias: {
      '@': '/src',
    },
  },
})
