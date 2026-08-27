import { ref } from "vue"
import { listen } from "@tauri-apps/api/event"
import { fetchDiskInfo, startScan } from "@/api/cleaner"
import { useScanControl } from "@/composables/useScanControl"
import type { DiskInfo, ScanItem, ScanMode, ScanProgress } from "@/types/cleaner"
import { isTauriRuntime } from "@/utils/tauriPlatform"

const diskInfo = ref<DiskInfo | null>(null)
const loading = ref(false)
const scanning = ref(false)
const scanProgress = ref<ScanProgress | null>(null)
const scanItems = ref<ScanItem[]>([])
const lastScanMode = ref<ScanMode>("quick")
const lastScanAt = ref<number | null>(null)

const { scanPaused, resetScanControlState, resetScanBackend, pauseScan, resumeScan } = useScanControl()

let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

function applyScanResult(items: ScanItem[], mode: ScanMode) {
  scanItems.value = items.map((item) => ({ ...item, selected: item.risk === "low" }))
  lastScanMode.value = mode
  lastScanAt.value = Date.now()
}

export function useDiskInfo() {
  const load = async () => {
    loading.value = true
    try {
      diskInfo.value = await fetchDiskInfo()
    } finally {
      loading.value = false
    }
  }

  return { diskInfo, loading, load }
}

export function useScanner() {
  const setupListeners = async () => {
    if (!isTauriRuntime()) return

    unlistenProgress = await listen<ScanProgress>("scan_progress", (event) => {
      scanProgress.value = event.payload
    })

    unlistenComplete = await listen<ScanItem[]>("scan_complete", (event) => {
      applyScanResult(event.payload, lastScanMode.value)
      scanning.value = false
      resetScanControlState()
    })
  }

  const teardownListeners = () => {
    unlistenProgress?.()
    unlistenComplete?.()
    unlistenProgress = null
    unlistenComplete = null
  }

  const scan = async (mode: ScanMode = "quick") => {
    lastScanMode.value = mode
    scanning.value = true
    scanProgress.value = null
    scanItems.value = []
    await resetScanBackend()

    if (isTauriRuntime()) {
      await setupListeners()
    }

    try {
      const result = await startScan(mode)
      if (!isTauriRuntime()) {
        applyScanResult(result.items, mode)
        scanProgress.value = {
          phase: "complete",
          current_path: "扫描完成",
          items_found: result.items.length,
          bytes_found: result.total_bytes,
          percent: 100,
        }
      } else if (scanning.value) {
        applyScanResult(result.items, mode)
        scanning.value = false
        resetScanControlState()
      }
    } catch (e) {
      scanning.value = false
      resetScanControlState()
      throw e
    } finally {
      if (!isTauriRuntime()) {
        scanning.value = false
      }
    }
  }

  const rescan = async () => {
    await scan(lastScanMode.value)
  }

  const reset = () => {
    scanItems.value = []
    scanProgress.value = null
    scanning.value = false
    lastScanAt.value = null
    resetScanControlState()
  }

  return {
    scanning,
    scanPaused,
    scanProgress,
    scanItems,
    lastScanMode,
    lastScanAt,
    scan,
    rescan,
    reset,
    pauseScan,
    resumeScan,
    teardownListeners,
  }
}
