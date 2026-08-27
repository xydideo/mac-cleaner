import { createRouter, createWebHashHistory } from "vue-router"
import type { RouteRecordRaw } from "vue-router"

export const appRoutes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "Dashboard",
    component: () => import("@/views/DashboardView/index.vue"),
    meta: { title: "概览", icon: "Odometer" },
  },
  {
    path: "/junk",
    name: "JunkClean",
    component: () => import("@/views/JunkCleanView/index.vue"),
    meta: { title: "垃圾清理", icon: "Delete" },
  },
  {
    path: "/uninstall",
    name: "AppUninstall",
    component: () => import("@/views/AppUninstallView/index.vue"),
    meta: { title: "应用卸载", icon: "Box" },
  },
  {
    path: "/folder",
    name: "CustomFolder",
    component: () => import("@/views/CustomFolderView/index.vue"),
    meta: { title: "自定义目录", icon: "FolderOpened" },
  },
  {
    path: "/large-files",
    name: "LargeFileScan",
    component: () => import("@/views/LargeFileScanView/index.vue"),
    meta: { title: "大文件", icon: "Search" },
  },
  {
    path: "/settings",
    name: "Settings",
    component: () => import("@/views/SettingsView/index.vue"),
    meta: { title: "设置", icon: "Setting" },
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes: appRoutes,
})

export default router
