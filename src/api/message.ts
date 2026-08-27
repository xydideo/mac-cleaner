import { invoke } from "@tauri-apps/api/core"
import { isTauriRuntime } from "@/utils/tauriPlatform"

export interface TauriMessageResponse<T> {
  type: string
  data: T
}

/** 统一 Tauri 消息通道，避免各 API 文件重复 invoke 样板代码 */
export async function invokeMessage<T>(
  type: string,
  data: Record<string, unknown> = {}
): Promise<T> {
  if (!isTauriRuntime()) {
    throw new Error("仅桌面端可用")
  }
  const result = await invoke<TauriMessageResponse<T>>("tauri_message", { type, data })
  return result.data
}

export async function invokeMessageOptional<T>(
  type: string,
  data: Record<string, unknown> = {},
  fallback: T
): Promise<T> {
  if (!isTauriRuntime()) return fallback
  return invokeMessage<T>(type, data)
}
