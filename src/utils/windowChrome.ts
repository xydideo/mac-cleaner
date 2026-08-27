import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { LogicalSize, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi"
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window"
import { isMacTauri, isTauriRuntime } from "@/utils/tauriPlatform"

const RESTORE_WIDTH = 1100
const RESTORE_HEIGHT = 720
const HEADER_DRAG_DELAY_MS = 240

export const windowMaximized = ref(false)
export const windowHeightMaximized = ref(false)

interface SavedWindowBounds {
  width: number
  height: number
  x: number
  y: number
}

let listening = false
let ignoreResizeSync = false
let headerDragTimer: ReturnType<typeof setTimeout> | null = null
let savedHeightBounds: SavedWindowBounds | null = null

function applyMaximizedClass(maximized: boolean) {
  windowMaximized.value = maximized
  document.documentElement.classList.toggle("window-maximized", maximized)
}

function applyHeightMaximizedClass(maximized: boolean) {
  windowHeightMaximized.value = maximized
  document.documentElement.classList.toggle("window-height-maximized", maximized)
}

function clearHeightMaximizedState() {
  savedHeightBounds = null
  applyHeightMaximizedClass(false)
}

function cancelHeaderDragTimer() {
  if (headerDragTimer) {
    clearTimeout(headerDragTimer)
    headerDragTimer = null
  }
}

async function readWindowBoundsPhysical(
  win: ReturnType<typeof getCurrentWindow>
): Promise<SavedWindowBounds> {
  const outerSize = await win.outerSize()
  const outerPos = await win.outerPosition()

  return {
    width: outerSize.width,
    height: outerSize.height,
    x: outerPos.x,
    y: outerPos.y,
  }
}

async function toggleWindowHeightMaximizedJs(): Promise<void> {
  const win = getCurrentWindow()

  try {
    if (await win.isMaximized()) {
      await toggleWindowMaximized()
      return
    }
  } catch {
    // ignore
  }

  ignoreResizeSync = true

  try {
    if (windowHeightMaximized.value && savedHeightBounds) {
      const bounds = savedHeightBounds
      savedHeightBounds = null
      await win.setSize(new PhysicalSize(bounds.width, bounds.height))
      await win.setPosition(new PhysicalPosition(bounds.x, bounds.y))
      applyHeightMaximizedClass(false)
      return
    }

    const [current, monitor, innerSize, outerSize] = await Promise.all([
      readWindowBoundsPhysical(win),
      currentMonitor(),
      win.innerSize(),
      win.outerSize(),
    ])
    if (!monitor) return

    const workArea = monitor.workArea
    const chromeHeight = Math.max(0, outerSize.height - innerSize.height)
    const targetOuterHeight = workArea.size.height
    const targetInnerHeight = Math.max(600, targetOuterHeight - chromeHeight)

    if (
      Math.abs(current.height - targetOuterHeight) <= 2 &&
      Math.abs(current.y - workArea.position.y) <= 2
    ) {
      return
    }

    savedHeightBounds = current
    await win.setSize(new PhysicalSize(innerSize.width, targetInnerHeight))
    await win.setPosition(new PhysicalPosition(current.x, workArea.position.y))
    applyHeightMaximizedClass(true)
  } finally {
    setTimeout(() => {
      ignoreResizeSync = false
    }, 300)
  }
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
      clearHeightMaximizedState()
    }
    return windowMaximized.value
  } catch {
    return windowMaximized.value
  }
}

export async function toggleWindowMaximized(): Promise<void> {
  if (!isTauriRuntime()) return

  cancelHeaderDragTimer()

  if (isMacTauri()) {
    try {
      await invoke("reset_window_height_saved")
    } catch {
      // ignore
    }
  }

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
      clearHeightMaximizedState()
    } else {
      clearHeightMaximizedState()
      await win.maximize()
      applyMaximizedClass(true)
    }
  } finally {
    setTimeout(() => {
      ignoreResizeSync = false
      syncWindowMaximized()
    }, 300)
  }
}

/** 双击标题栏：仅拉高窗口至当前屏幕可用高度，宽度与水平位置不变 */
export async function toggleWindowHeightMaximized(): Promise<void> {
  if (!isTauriRuntime()) return

  cancelHeaderDragTimer()

  if (isMacTauri()) {
    try {
      const maximized = await invoke<boolean>("toggle_window_height_maximized")
      applyHeightMaximizedClass(maximized)
      if (!maximized) {
        savedHeightBounds = null
      }
      return
    } catch {
      // 回退到 JS 方案
    }
  }

  await toggleWindowHeightMaximizedJs()
}

/** 标题栏按下：延迟拖动，避免吃掉双击 */
export function handleHeaderMouseDown(event: MouseEvent): void {
  if (!isTauriRuntime()) return

  const target = event.target as HTMLElement | null
  if (target?.closest(".traffic-lights")) return

  if (event.detail >= 2) {
    cancelHeaderDragTimer()
    event.preventDefault()
    void toggleWindowHeightMaximized()
    return
  }

  if (event.detail !== 1) return

  // 阻止拖动窗口时误触选中文本（WebKit 侧滑/选中）
  event.preventDefault()

  cancelHeaderDragTimer()
  headerDragTimer = setTimeout(() => {
    headerDragTimer = null
    void getCurrentWindow()
      .startDragging()
      .catch(() => {
        // ignore
      })
  }, HEADER_DRAG_DELAY_MS)
}

export async function ensureWindowMaximizeListener(): Promise<void> {
  if (!isTauriRuntime() || listening) return

  listening = true
  document.documentElement.classList.add("desktop-window")
  await syncWindowMaximized()

  await getCurrentWindow().onResized(() => {
    if (ignoreResizeSync) return
    syncWindowMaximized()
  })
}
