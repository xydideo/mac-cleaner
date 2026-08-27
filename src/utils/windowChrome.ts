import { ref } from "vue"
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window"
import { isTauriRuntime } from "@/utils/tauriPlatform"

const RESTORE_WIDTH = 1100
const RESTORE_HEIGHT = 720

export const windowMaximized = ref(false)

let listening = false
let ignoreResizeSync = false

function applyMaximizedClass(maximized: boolean) {
  windowMaximized.value = maximized
  document.documentElement.classList.toggle("window-maximized", maximized)
}

export async function syncWindowMaximized(): Promise<boolean> {
  if (!isTauriRuntime()) {
    applyMaximizedClass(false)
    return false
  }

  if (ignoreResizeSync) {
    return windowMaximized.value
  }

  try {
    const maximized = await getCurrentWindow().isMaximized()
    if (maximized) {
      applyMaximizedClass(true)
    }
    return windowMaximized.value
  } catch {
    return windowMaximized.value
  }
}

export async function toggleWindowMaximized(): Promise<void> {
  if (!isTauriRuntime()) return

  const win = getCurrentWindow()
  let maximized = windowMaximized.value
  try {
    if (await win.isMaximized()) {
      maximized = true
    }
  } catch {
    // ignore
  }

  ignoreResizeSync = true

  try {
    if (maximized) {
      try {
        await win.unmaximize()
      } catch {
        // ignore
      }
      await win.setSize(new LogicalSize(RESTORE_WIDTH, RESTORE_HEIGHT))
      try {
        await win.center()
      } catch {
        // ignore
      }
      applyMaximizedClass(false)
    } else {
      await win.maximize()
      applyMaximizedClass(true)
    }
  } finally {
    setTimeout(() => {
      ignoreResizeSync = false
      syncWindowMaximized()
    }, 200)
  }
}

export async function ensureWindowMaximizeListener(): Promise<void> {
  if (!isTauriRuntime() || listening) return

  listening = true
  document.documentElement.classList.add("desktop-window")
  await syncWindowMaximized()

  await getCurrentWindow().onResized(() => {
    syncWindowMaximized()
  })
}
