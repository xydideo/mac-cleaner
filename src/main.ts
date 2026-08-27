import { createApp } from "vue"
import App from "./App.vue"
import router from "./router"
import ElementPlus from "element-plus"
import zhCn from "element-plus/dist/locale/zh-cn.mjs"
import "element-plus/theme-chalk/src/index.scss"
import "./assets/styles/reset-element-var.scss"
import "./assets/styles/index.scss"
import { APP_TITLE } from "@/constants/app"
import { isTauriRuntime } from "@/utils/tauriPlatform"

document.title = APP_TITLE

if (isTauriRuntime()) {
  document.documentElement.classList.add("desktop-window")
}

const app = createApp(App)
app.use(ElementPlus, { locale: zhCn })
app.use(router)
app.mount("#app")
