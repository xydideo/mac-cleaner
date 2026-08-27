export interface DiskInfo {
  total_bytes: number
  used_bytes: number
  available_bytes: number
  usage_percent: number
  mount_point: string
}

export interface BrowseItem {
  id: string
  path: string
  name: string
  is_directory: boolean
  size_bytes: number
  size_ready: boolean
  /** 深扫未完整统计（超过 5GB 或 20 万文件）时为 true，展示为 >已扫描体积 */
  size_capped?: boolean
  modified: string
  risk: "low" | "medium" | "high"
  risk_label: string
  description: string
  junk_bytes: number
  junk_count: number
  child_count: number
  /** 系统保护路径，不可删除 */
  protected?: boolean
  /** 所属应用展示标签，如「Chrome 应用」 */
  app_label?: string
  bundle_id?: string
  app_installed?: boolean
  selected?: boolean
}

export interface FolderBrowseSummary {
  path: string
  total_bytes: number
  folder_count: number
  file_count: number
  junk_bytes: number
  junk_count: number
  pending_count?: number
}

export interface LargeFileScanResult {
  items: BrowseItem[]
  summary: FolderBrowseSummary
  files_scanned: number
}

export interface ScanItem {
  id: string
  path: string
  size_bytes: number
  category: string
  risk: "low" | "medium" | "high"
  title: string
  description: string
  last_modified: string
  app_name?: string
  app_label?: string
  bundle_id?: string
  app_installed?: boolean
  selected?: boolean
}

export interface ScanProgress {
  phase: string
  current_path: string
  items_found: number
  bytes_found: number
  percent: number
}

export interface CleanResult {
  success: string[]
  failed: { path: string; message: string }[]
  total_freed_bytes: number
}

export interface AppInfo {
  name: string
  path: string
  bundle_id: string
  version: string
  size_bytes: number
  icon_base64?: string
  is_system_app: boolean
}

export interface AppRelatedItem {
  id: string
  path: string
  size_bytes: number
  category: string
  risk: "low" | "medium" | "high"
  title: string
  description: string
  match_type: string
  selected?: boolean
}

export type ScanMode = "quick" | "deep"

export type NavItem = {
  path: string
  title: string
  icon: string
}
