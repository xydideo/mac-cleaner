import type { Router } from "vue-router"

/** 跳转到自定义目录模块并携带路径，进入后自动扫描 */
export function navigateToFolderScan(
  router: Router,
  path: string,
  options?: { highlight?: string }
) {
  const query: Record<string, string> = { path }
  if (options?.highlight) {
    query.highlight = options.highlight
  }
  router.push({ name: "CustomFolder", query })
}
