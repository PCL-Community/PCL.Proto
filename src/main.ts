import 'animate.css'
import './assets/main.css'

import App from './App.vue'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from '@/router/index'
import { createI18n } from 'vue-i18n'
import locales from './locales'

const pinia = createPinia()

const i18n = createI18n({
    // locale: navigator.language,
    fallbackLocale: 'en-US',
    messages: locales
})

const app = createApp(App)

app.use(router)
app.use(pinia)
app.use(i18n)

app.mount('#app')
