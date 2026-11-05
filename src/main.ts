/**
 ____   ____ _       ____            _        
|  _ \ / ___| |     |  _ \ _ __ ___ | |_ ___  
| |_) | |   | |     | |_) | '__/ _ \| __/ _ \ 
|  __/| |___| |___ _|  __/| | | (_) | || (_) |
|_|    \____|_____(_)_|   |_|  \___/ \__\___/ 
 */
import './assets/main.css'
import { isTauri } from '@tauri-apps/api/core'
import { initializeDownloadSettings } from './api/downloadSettings'
import { initializeLogSettings } from './api/logSettings'
import { initializeWindowSettings } from './api/windowSettings'

import App from './App.vue'
import { initializePclLocale, pclText } from './api/pclLocale'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router/index'
import { useAccountInfo } from './stores/account'
import { attachConsole } from '@tauri-apps/plugin-log'
import { useSelectedInstance } from './stores/gameLaunch'
import { installExternalLinkHandler } from './util/externalLink'
import './api/appearance'
import { checkLauncherUpdate, launcherUpdate } from './api/launcherUpdate'
import { useModal } from './composables/useModal'

const pinia = createPinia()

const app = createApp(App)

app.use(pinia)
app.use(router)

// ignore non-emit warnnings
if (import.meta.env.DEV) {
  app.config.warnHandler = (msg, instance, trace) => {
    if (msg.includes('Extraneous non-emits event listeners')) {
      return
    }
    console.warn(msg, trace)
  }
}

document.documentElement.classList.add('theme-blue')
document.documentElement.classList.toggle('website', !isTauri())
const removeExternalLinkHandler = installExternalLinkHandler()
import.meta.hot?.dispose(removeExternalLinkHandler)

void initializePclLocale().then(async () => {
  if (isTauri()) {
    try {
      await initializeDownloadSettings()
      await initializeLogSettings()
      await initializeWindowSettings()
    } catch (error) {
      const root = document.querySelector('#app')!
      root.textContent = `Unable to load settings: ${String(error)}`
      const retry = document.createElement('button')
      retry.textContent = 'Retry / 重试'
      retry.onclick = () => location.reload()
      root.appendChild(retry)
      return
    }
  }
  app.mount('#app')
  if (isTauri()) {
    void checkLauncherUpdate().then(() => {
      const update = launcherUpdate.value
      if (!update || update.version.replace(/^v/, '') === __APP_VERSION__) return

      const modal = useModal()
      void modal.open({
        title: pclText('Update.Title'),
        content: pclText('Update.Available', [__APP_VERSION__, update.version]),
        buttons: [
          {
            content: pclText('Setup.Update.Changelog.View'),
            type: 'tint',
            operation: () => {
              modal.close(true)
              void router.push('/setup/update')
            },
          },
          { content: pclText('Common.Action.Cancel'), operation: () => modal.close(false) },
        ],
      })
    })
  }
})

// some async actions
if (isTauri()) void useAccountInfo().initialize()
if (isTauri()) {
  void attachConsole()
  void useSelectedInstance().fetch().catch(console.error)
}
