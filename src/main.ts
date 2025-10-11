import 'animate.css'
import './assets/main.css'

import App from './App.vue'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router/index'
import { createI18n } from 'vue-i18n'
import locales from './locales'
import { useAccountInfo } from './stores/account'
import { attachConsole } from '@tauri-apps/plugin-log'

const pinia = createPinia()

const i18n = createI18n({
    legacy: false,
    locale: navigator.language,
    fallbackLocale: 'en-US',
    messages: locales
})

const app = createApp(App)

app.use(router)
app.use(pinia)
app.use(i18n)

async function initializeAppState() {
    const accountStore = useAccountInfo()
    await accountStore.initialize()
    await attachConsole();
}

app.mount('#app')
initializeAppState()
