import { invoke } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import type { BrowseItem, FolderBrowseSummary, LargeFileScanResult } from "@/types/cleaner"

export interface FolderShortcut {
  id: string
  label: string
  path: string
  description: string
  action?: "large_file_scan" | "settings_large_file"
}

export async function fetchFolderShortcuts(): Promise<FolderShortcut[]> {
  if (!isTauriRuntime()) {
    return []
  }
  const result = await invoke<{ type: string; data: FolderShortcut[] }>("tauri_message", {
    type: "folder_shortcuts",
    data: {},
  })
  return result.data
}

export async function browseFolder(
  path: string,
  options: { showHidden?: boolean } = {}
): Promise<{ items: BrowseItem[]; summary: FolderBrowseSummary }> {
  if (!isTauriRuntime()) {
    return {
      items: [],
      summary: {
        path,
        total_bytes: 0,
        folder_count: 0,
        file_count: 0,
        junk_bytes: 0,
        junk_count: 0,
        pending_count: 0,
      },
    }
  }
  const result = await invoke<{
    type: string
    data: { items: BrowseItem[]; summary: FolderBrowseSummary }
  }>("tauri_message", {
    type: "folder_browse",
    data: { path, show_hidden: options.showHidden ?? false },
  })
  return result.data
}

export async function analyzeFolderItems(
  paths: string[],
  options: { exact?: boolean } = {}
): Promise<{ items: BrowseItem[] }> {
  if (!isTauriRuntime() || paths.length === 0) {
    return { items: [] }
  }
  const result = await invoke<{
    type: string
    data: { items: BrowseItem[] }
  }>("tauri_message", {
    type: "folder_analyze",
    data: { paths, exact: options.exact ?? false },
  })
  return result.data
}

export async function scanLargeFiles(
  minFileBytes: number,
  minFolderBytes: number
): Promise<LargeFileScanResult> {
  if (!isTauriRuntime()) {
    return {
      items: [],
      files_scanned: 0,
      summary: {
        path: "large-file-scan://",
        total_bytes: 0,
        folder_count: 0,
        file_count: 0,
        junk_bytes: 0,
        junk_count: 0,
        pending_count: 0,
      },
    }
  }
  const result = await invoke<{
    type: string
    data: LargeFileScanResult
  }>("tauri_message", {
    type: "large_file_scan",
    data: { min_file_bytes: minFileBytes, min_folder_bytes: minFolderBytes },
  })
  return result.data
}

/** @deprecated 使用 browseFolder */
export async function scanFolder(path: string) {
  const { items, summary } = await browseFolder(path)
  return { items, total_bytes: summary.total_bytes }
}
