import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import './assets/css/tailwind.css'

import { router } from './router'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(router)

app.mount('#app')
