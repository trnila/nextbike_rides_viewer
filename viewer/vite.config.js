import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/rides': 'http://localhost:8080/',
      '/stations.json': 'http://localhost:8080/',
    }
  }
})
