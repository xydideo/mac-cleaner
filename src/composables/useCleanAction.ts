import { ref } from "vue"
import { ElMessage, ElMessageBox } from "element-plus"
import { cleanPaths, type CleanMode } from "@/api/cleaner"
import { formatSize } from "@/utils/formatSize"
import { notifyCleanResult } from "@/utils/cleanResult"
import type { CleanResult } from "@/types/cleaner"

export interface CleanTarget {
  path: string
  size_bytes: number
}

export interface CleanConfirmOptions {
  targets: CleanTarget[]
  mode: CleanMode
  title: string
  message: string
  confirmButtonText: string
  /** 永久删除等需要二次确认时使用 */
  secondConfirm?: {
    title: string
    message: string
    confirmButtonText: string
  }
  actionLabel?: string
  onSuccess?: (result: CleanResult) => void | Promise<void>
}

/** 统一的删除确认 + 执行 + 结果通知 */
export function useCleanAction() {
  const cleaning = ref(false)
  const cleaningText = ref("")

  const confirmClean = async (options: CleanConfirmOptions) => {
    const { targets, mode, title, message, confirmButtonText, secondConfirm } = options
    if (!targets.length) return false

    await ElMessageBox.confirm(message, title, {
      confirmButtonText,
      cancelButtonText: "取消",
      type: mode === "permanent" ? "warning" : "warning",
    })

    if (secondConfirm) {
      await ElMessageBox.confirm(secondConfirm.message, secondConfirm.title, {
        confirmButtonText: secondConfirm.confirmButtonText,
        cancelButtonText: "取消",
        type: "error",
        confirmButtonClass: "el-button--danger",
      })
    }

    return true
  }

  const executeClean = async (options: CleanConfirmOptions) => {
    const { targets, mode, actionLabel, onSuccess } = options
    if (!targets.length) return

    cleaning.value = true
    cleaningText.value =
      actionLabel ?? (mode === "trash" ? "正在移到废纸篓…" : "正在永久删除…")

    try {
      const paths = targets.map((item) => item.path)
      const sizes = targets.map((item) => item.size_bytes)
      const result = await cleanPaths(paths, mode, sizes)
      await notifyCleanResult(result, mode === "trash" ? "移到废纸篓" : "永久删除")
      await onSuccess?.(result)
      return result
    } catch (error) {
      ElMessage.error(String(error))
      return null
    } finally {
      cleaning.value = false
      cleaningText.value = ""
    }
  }

  const cleanWithConfirm = async (options: CleanConfirmOptions) => {
    try {
      const confirmed = await confirmClean(options)
      if (!confirmed) return null
      return executeClean(options)
    } catch {
      return null
    }
  }

  const formatTargetsSummary = (targets: CleanTarget[]) => {
    const bytes = targets.reduce((sum, item) => sum + item.size_bytes, 0)
    return `${targets.length} 项（${formatSize(bytes)}）`
  }

  return {
    cleaning,
    cleaningText,
    confirmClean,
    executeClean,
    cleanWithConfirm,
    formatTargetsSummary,
  }
}
