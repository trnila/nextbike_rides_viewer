import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { viteSingleFile } from "vite-plugin-singlefile";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    svelte(),
    mode !== "development" && viteSingleFile(),
  ],
  server: {
    proxy: {
      '/rides': 'http://localhost:8080/',
      '/stations.json': 'http://localhost:8080/',
    }
  }
}))
