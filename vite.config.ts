import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { version } from './package.json'
import svgLoader from 'vite-svg-loader'
// import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  base: '/PCL.Proto/',
  plugins: [
    vue(),
    svgLoader()
    // vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(version),
  }
})
