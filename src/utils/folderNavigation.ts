import type { Router } from "vue-router"

/** 跳转到自定义目录模块并携带路径，进入后自动扫描 */
export function navigateToFolderScan(router: Router, path: string) {
  router.push({ name: "CustomFolder", query: { path } })
}
