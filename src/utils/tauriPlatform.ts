export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
}

export function isMacTauri(): boolean {
  if (!isTauriRuntime()) return false
  return /mac/i.test(navigator.platform) || /mac/i.test(navigator.userAgent)
}
