import { fileURLToPath, URL } from 'node:url'
import { execFileSync } from 'node:child_process'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import packageJson from './package.json' with { type: 'json' }
import svgLoader from 'vite-svg-loader'
import vueJsx from '@vitejs/plugin-vue-jsx'
// import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
let appVersion = packageJson.version
if (process.env.PCL_RELEASE !== 'true') {
  let revision = 'unknown'
  try {
    revision = execFileSync('git', ['rev-parse', '--short=8', 'HEAD'], {
      cwd: fileURLToPath(new URL('.', import.meta.url)),
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim()
  } catch {
    // 源码压缩包可能不包含 Git 信息。
  }
  appVersion += `${appVersion.includes('-') ? '.dev' : '-dev'}+${revision}`
}

export default defineConfig({
  base: process.env.TAURI_ENV_PLATFORM ? '/' : '/PCL.Proto/',
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // Rust 编译产物位于工作区根目录，避免启动时遍历整个 target。
      ignored: ['**/target/**', '**/src-tauri/**', '**/crates/terracotta/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  optimizeDeps: { rolldownOptions: { transform: { jsx: 'preserve' } } },
  build: {
    // Tauri 在 Windows 上使用 Chromium，在 macOS 和 Linux 上使用 WebKit
    target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // 在 debug 构建中不使用 minify
    minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
    // 在 debug 构建中生成 sourcemap
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  plugins: [
    vue(),
    svgLoader(),
    vueJsx(),
    // vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
  },
})
