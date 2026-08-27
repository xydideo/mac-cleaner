const UNITS = ["B", "KB", "MB", "GB", "TB"] as const

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B"
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), UNITS.length - 1)
  const value = bytes / 1024 ** i
  return `${value >= 100 || i === 0 ? value.toFixed(0) : value.toFixed(1)} ${UNITS[i]}`
}

/** 目录浏览项体积展示（含计算中与未完整统计时的 > 前缀） */
export function formatBrowseSize(item: {
  size_ready: boolean
  size_capped?: boolean
  size_bytes: number
}): string {
  if (!item.size_ready) return "计算中…"
  const label = formatSize(item.size_bytes)
  if (item.size_capped) return `>${label}`
  return label
}

/** 汇总体积：存在未完整统计项时加 > 前缀 */
export function formatAggregateSize(
  bytes: number,
  options: { hasCapped?: boolean } = {}
): string {
  const label = formatSize(bytes)
  if (options.hasCapped) return `>${label}`
  return label
}

export function formatPercent(value: number): string {
  return `${Math.round(value)}%`
}
