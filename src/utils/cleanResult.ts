import { ElMessage, ElMessageBox } from "element-plus"
import type { CleanResult } from "@/types/cleaner"
import { formatSize } from "@/utils/formatSize"

function withTimeout<T>(promise: Promise<T>, ms: number, message: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(message)), ms)
    promise
      .then((value) => {
        clearTimeout(timer)
        resolve(value)
      })
      .catch((err) => {
        clearTimeout(timer)
        reject(err)
      })
  })
}

/** 根据路径数量估算删除超时（每项约 25 秒 + 10 秒缓冲） */
export function cleanTimeoutMs(pathCount: number): number {
  return Math.max(30_000, pathCount * 25_000 + 10_000)
}

export async function showCleanFailures(failed: CleanResult["failed"]) {
  if (failed.length === 0) return

  const lines = failed
    .slice(0, 8)
    .map((f) => {
      const name = f.path.split("/").pop() || f.path
      return `• ${name}：${f.message}`
    })
    .join("\n")

  const tail = failed.length > 8 ? `\n…等共 ${failed.length} 项失败` : ""

  await ElMessageBox.alert(`${lines}${tail}`, `${failed.length} 项未能删除`, {
    type: "warning",
    confirmButtonText: "知道了",
  })
}

export async function notifyCleanResult(result: CleanResult, actionLabel = "处理") {
  const { success, failed, total_freed_bytes } = result

  if (failed.length === 0) {
    ElMessage.success(
      `已${actionLabel} ${success.length} 项，释放 ${formatSize(total_freed_bytes)}`
    )
    return
  }

  if (success.length === 0) {
    ElMessage.error(`${actionLabel}失败，${failed.length} 项均未删除`)
    await showCleanFailures(failed)
    return
  }

  ElMessage.warning(
    `已${actionLabel} ${success.length} 项，${failed.length} 项失败，释放 ${formatSize(total_freed_bytes)}`
  )
  await showCleanFailures(failed)
}

export { withTimeout }
