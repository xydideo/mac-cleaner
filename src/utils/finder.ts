import { invoke } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"

/** 在访达中显示并选中指定路径 */
export async function revealInFinder(path: string): Promise<void> {
  if (!isTauriRuntime()) return
  await invoke("reveal_in_finder", { path })
}
