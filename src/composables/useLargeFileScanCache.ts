import { ref } from "vue"
import type { BrowseItem } from "@/types/cleaner"
import type { LargeScanKind } from "@/api/folder"

export type LargeScanItem = BrowseItem & { selected?: boolean }

export interface LargeTabScanState {
  items: LargeScanItem[]
  totalBytes: number
  filesScanned: number
  scannedAt: Date | null
  scannedOnce: boolean
}

const createTabState = (): LargeTabScanState => ({
  items: [],
  totalBytes: 0,
  filesScanned: 0,
  scannedAt: null,
  scannedOnce: false,
})

/** 模块级缓存：切换页面后保留大文件/大文件夹扫描结果 */
const activeTab = ref<LargeScanKind>("folders")
const folderState = ref<LargeTabScanState>(createTabState())
const fileState = ref<LargeTabScanState>(createTabState())

export function useLargeFileScanCache() {
  const tabState = (kind: LargeScanKind) => (kind === "folders" ? folderState : fileState)

  const resetTabState = (kind: LargeScanKind) => {
    tabState(kind).value = createTabState()
  }

  const resetAll = () => {
    folderState.value = createTabState()
    fileState.value = createTabState()
    activeTab.value = "folders"
  }

  return {
    activeTab,
    folderState,
    fileState,
    tabState,
    resetTabState,
    resetAll,
  }
}
