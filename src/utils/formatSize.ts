const UNITS = ["B", "KB", "MB", "GB", "TB"] as const

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B"
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), UNITS.length - 1)
  const value = bytes / 1024 ** i
  return `${value >= 100 || i === 0 ? value.toFixed(0) : value.toFixed(1)} ${UNITS[i]}`
}

/** 目录浏览项体积展示（含计算中与 >5G 封顶） */
export function formatBrowseSize(item: {
  size_ready: boolean
  size_capped?: boolean
  size_bytes: number
}): string {
  if (!item.size_ready) return "计算中…"
  if (item.size_capped) return ">5G"
  return formatSize(item.size_bytes)
}

export function formatPercent(value: number): string {
  return `${Math.round(value)}%`
}
