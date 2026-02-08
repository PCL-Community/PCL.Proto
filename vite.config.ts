import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { version } from './package.json'
import svgLoader from 'vite-svg-loader'
import vueJsx from '@vitejs/plugin-vue-jsx'
// import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  base: process.env.TAURI_ENV_PLATFORM ? '/' : '/PCL.Proto/',
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**', '**/src-terracotta/**'],
    }
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // Tauri 在 Windows 上使用 Chromium，在 macOS 和 Linux 上使用 WebKit
    target:
      process.env.TAURI_ENV_PLATFORM == 'windows'
        ? 'chrome105'
        : 'safari13',
    // 在 debug 构建中不使用 minify
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    // 在 debug 构建中生成 sourcemap
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  plugins: [
    vue(),
    svgLoader(),
    vueJsx()
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
