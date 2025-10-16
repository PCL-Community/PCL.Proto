import './assets/main.css'

import App from './App.vue'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router/index'
import { createI18n } from 'vue-i18n'
import locales from './locales/src'
import { useAccountInfo } from './stores/account'
import { attachConsole } from '@tauri-apps/plugin-log'
import { useSelectedInstance } from './stores/gameLaunch'
import { vAnimateChildrenDrop, vAnimateDrop } from './util/cardDropAnimate'

const pinia = createPinia()

const i18n = createI18n({
  legacy: false,
  locale: navigator.language,
  fallbackLocale: 'en-US',
  messages: locales,
})

const app = createApp(App)

app.use(pinia)
app.use(i18n)
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

// global card drop animate
app.directive('card-drop-animate', vAnimateDrop)
app.directive('card-drop-children-animate', vAnimateChildrenDrop)

app.mount('#app')

// some async actions
useAccountInfo().initialize()
attachConsole()
useSelectedInstance().fetch()
