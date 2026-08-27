import type { Component } from "vue"
import {
  Box,
  Delete,
  FolderOpened,
  Odometer,
  Search,
  Setting,
} from "@element-plus/icons-vue"
import { markRaw } from "vue"

export interface NavItem {
  path: string
  title: string
  icon: Component
  sidebar?: boolean
}

/** 侧边栏与路由共享的导航配置（单一数据源） */
export const mainNavItems: NavItem[] = [
  { path: "/", title: "概览", icon: markRaw(Odometer), sidebar: true },
  { path: "/junk", title: "清理", icon: markRaw(Delete), sidebar: true },
  { path: "/large-files", title: "大文件", icon: markRaw(Search), sidebar: true },
  { path: "/uninstall", title: "APP", icon: markRaw(Box), sidebar: true },
  { path: "/folder", title: "目录", icon: markRaw(FolderOpened), sidebar: true },
]

export const settingsNavItem: NavItem = {
  path: "/settings",
  title: "设置",
  icon: markRaw(Setting),
  sidebar: false,
}

export const sidebarNavItems = mainNavItems.filter((item) => item.sidebar)
