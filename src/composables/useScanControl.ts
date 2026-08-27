import { ref } from "vue"
import { controlScan, type ScanControlAction } from "@/api/scanControl"

const scanPaused = ref(false)

export function useScanControl() {
  const resetScanControlState = () => {
    scanPaused.value = false
  }

  const resetScanBackend = async () => {
    await controlScan("reset")
    resetScanControlState()
  }

  const runScanControl = async (action: ScanControlAction) => {
    await controlScan(action)
    if (action === "pause") scanPaused.value = true
    if (action === "resume") scanPaused.value = false
    if (action === "cancel") scanPaused.value = false
  }

  const pauseScan = () => runScanControl("pause")
  const resumeScan = () => runScanControl("resume")
  const cancelScan = () => runScanControl("cancel")

  return {
    scanPaused,
    resetScanControlState,
    resetScanBackend,
    pauseScan,
    resumeScan,
    cancelScan,
  }
}
