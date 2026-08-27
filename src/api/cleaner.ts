import { invoke } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import { cleanTimeoutMs, withTimeout } from "@/utils/cleanResult"
import type { CleanResult, DiskInfo, ScanItem, ScanMode } from "@/types/cleaner"

export async function fetchDiskInfo(): Promise<DiskInfo> {
  if (!isTauriRuntime()) {
    return mockDiskInfo()
  }
  return invoke<DiskInfo>("get_disk_usage")
}

export async function startScan(mode: ScanMode = "quick"): Promise<{ items: ScanItem[]; total_bytes: number }> {
  if (!isTauriRuntime()) {
    return { items: mockScanItems(), total_bytes: mockScanItems().reduce((s, i) => s + i.size_bytes, 0) }
  }
  const result = await invoke<{ type: string; data: { items: ScanItem[]; total_bytes: number } }>("tauri_message", {
    type: "scan",
    data: { mode },
  })
  return result.data
}

export type CleanMode = "trash" | "permanent"

export async function cleanPaths(
  paths: string[],
  mode: CleanMode = "trash",
  sizes?: number[]
): Promise<CleanResult> {
  if (!isTauriRuntime()) {
    return { success: paths, failed: [], total_freed_bytes: 0 }
  }
  const result = await withTimeout(
    invoke<{ type: string; data: CleanResult }>("tauri_message", {
      type: "clean",
      data: { paths, mode, sizes },
    }),
    cleanTimeoutMs(paths.length),
    "删除操作超时，可能已被系统拦截。请关闭系统提示后重试，或检查权限设置"
  )
  return result.data
}

function mockDiskInfo(): DiskInfo {
  return {
    total_bytes: 250 * 1024 ** 3,
    used_bytes: 230 * 1024 ** 3,
    available_bytes: 20 * 1024 ** 3,
    usage_percent: 92,
    mount_point: "/",
  }
}

function mockScanItems(): ScanItem[] {
  return [
    {
      id: "mock_1",
      path: "~/Library/Caches/com.example.app",
      size_bytes: 2.1 * 1024 ** 3,
      category: "cache",
      risk: "low",
      title: "示例应用缓存",
      description: "演示数据（浏览器模式）",
      last_modified: "2025-06-01",
    },
  ]
}
