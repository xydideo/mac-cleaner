import { invoke } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import type { AppInfo, AppRelatedItem } from "@/types/cleaner"

export async function fetchInstalledApps(): Promise<AppInfo[]> {
  if (!isTauriRuntime()) {
    return mockApps()
  }
  const result = await invoke<{ type: string; data: AppInfo[] }>("tauri_message", {
    type: "list_apps",
    data: {},
  })
  return result.data
}

export async function scanAppRelated(
  app: Pick<AppInfo, "path" | "bundle_id" | "name">
): Promise<{ items: AppRelatedItem[]; total_bytes: number }> {
  if (!isTauriRuntime()) {
    return { items: mockRelated(app.name), total_bytes: 500_000_000 }
  }
  const result = await invoke<{
    type: string
    data: { items: AppRelatedItem[]; total_bytes: number }
  }>("tauri_message", {
    type: "app_scan",
    data: {
      app_path: app.path,
      bundle_id: app.bundle_id,
      app_name: app.name,
    },
  })
  return result.data
}

function mockApps(): AppInfo[] {
  return [
    {
      name: "示例应用",
      path: "/Applications/Demo.app",
      bundle_id: "com.example.demo",
      version: "1.0.0",
      size_bytes: 120_000_000,
      is_system_app: false,
    },
  ]
}

function mockRelated(name: string): AppRelatedItem[] {
  return [
    {
      id: "mock_main",
      path: `/Applications/${name}.app`,
      size_bytes: 120_000_000,
      category: "main",
      risk: "low",
      title: "主程序",
      description: "应用程序本体",
      match_type: "bundle_id",
    },
  ]
}
