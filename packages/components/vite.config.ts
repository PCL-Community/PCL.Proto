import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import svgLoader from 'vite-svg-loader'

export default defineConfig({
  publicDir: false,
  plugins: [vue(), svgLoader()],
  resolve: { alias: { '@': fileURLToPath(new URL('../../src', import.meta.url)) } },
  build: {
    outDir: 'packages/components/dist',
    lib: {
      entry: 'packages/components/src/build.ts',
      formats: ['es'],
      fileName: 'index',
      cssFileName: 'style',
    },
    rolldownOptions: { external: (id) => id === 'vue' || id.startsWith('vue/') },
  },
})
