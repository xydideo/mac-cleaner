import { invoke } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"

export type ScanControlAction = "pause" | "resume" | "cancel" | "reset"

export async function controlScan(action: ScanControlAction) {
  if (!isTauriRuntime()) return
  await invoke("tauri_message", {
    type: "scan_control",
    data: { action },
  })
}
