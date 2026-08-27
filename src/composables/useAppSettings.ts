import { computed, ref, watch } from "vue"
import { APP_NAME } from "@/constants/app"

const STORAGE_KEY = `${APP_NAME}-settings`

const FILE_MB_OPTIONS = [50, 100, 200, 500] as const
const FOLDER_MB_OPTIONS = [512, 1024, 2048, 5120] as const

export interface AppSettings {
  deleteMode: "trash" | "permanent"
  staleDays: number
  /** 大文件阈值（MB） */
  largeFileMB: number
  /** 大文件夹阈值（MB，1024 = 1GB） */
  largeFolderMB: number
  /** 目录模块列出以 . 开头的项 */
  showHiddenFiles: boolean
}

const DEFAULTS: AppSettings = {
  deleteMode: "trash",
  staleDays: 90,
  largeFileMB: 200,
  largeFolderMB: 1024,
  showHiddenFiles: true,
}

export function formatThresholdMB(mb: number): string {
  if (mb >= 1024 && mb % 1024 === 0) return `${mb / 1024} GB`
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`
  return `${mb} MB`
}

function pickOption(value: number | undefined, options: readonly number[], fallback: number): number {
  return value !== undefined && options.includes(value) ? value : fallback
}

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULTS }
    const parsed = JSON.parse(raw) as Partial<AppSettings>
    return {
      deleteMode: parsed.deleteMode === "permanent" ? "permanent" : "trash",
      staleDays: pickOption(parsed.staleDays, [30, 90, 180] as const, DEFAULTS.staleDays),
      largeFileMB: pickOption(parsed.largeFileMB, FILE_MB_OPTIONS, DEFAULTS.largeFileMB),
      largeFolderMB: pickOption(parsed.largeFolderMB, FOLDER_MB_OPTIONS, DEFAULTS.largeFolderMB),
      showHiddenFiles: parsed.showHiddenFiles === true,
    }
  } catch {
    return { ...DEFAULTS }
  }
}

const settings = ref<AppSettings>(loadSettings())

watch(
  settings,
  (value) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value))
  },
  { deep: true }
)

export function useAppSettings() {
  const largeFileBytes = computed(() => settings.value.largeFileMB * 1024 * 1024)
  const largeFolderBytes = computed(() => settings.value.largeFolderMB * 1024 * 1024)

  const largeScanThresholdLabel = computed(() =>
    `只扫描文件 ≥ ${formatThresholdMB(settings.value.largeFileMB)} · 文件夹 ≥ ${formatThresholdMB(settings.value.largeFolderMB)}`
  )

  const resetSettings = () => {
    settings.value = { ...DEFAULTS }
  }

  return {
    settings,
    largeFileBytes,
    largeFolderBytes,
    largeScanThresholdLabel,
    formatThresholdMB,
    resetSettings,
    FILE_MB_OPTIONS,
    FOLDER_MB_OPTIONS,
  }
}
