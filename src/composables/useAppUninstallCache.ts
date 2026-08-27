import { fetchInstalledApps, scanAppRelated } from "@/api/uninstall"
import type { AppInfo, AppRelatedItem } from "@/types/cleaner"

const CACHE_TTL_MS = 60 * 60 * 1000

interface TimedCache<T> {
  data: T
  cachedAt: number
}

interface UninstallUiState {
  keyword: string
  selectedAppPath: string | null
  cachedAt: number
}

let appsCache: TimedCache<AppInfo[]> | null = null
const relatedCache = new Map<string, TimedCache<AppRelatedItem[]>>()
let uiState: UninstallUiState | null = null

function isFresh(cachedAt: number) {
  return Date.now() - cachedAt < CACHE_TTL_MS
}

function relatedCacheKey(app: Pick<AppInfo, "path" | "bundle_id">) {
  return `${app.bundle_id}::${app.path}`
}

export function useAppUninstallCache() {
  const loadApps = async (force = false): Promise<AppInfo[]> => {
    if (!force && appsCache && isFresh(appsCache.cachedAt)) {
      return appsCache.data
    }
    const data = await fetchInstalledApps()
    appsCache = { data, cachedAt: Date.now() }
    return data
  }

  const loadRelated = async (app: AppInfo, force = false): Promise<AppRelatedItem[]> => {
    const key = relatedCacheKey(app)
    const cached = relatedCache.get(key)
    if (!force && cached && isFresh(cached.cachedAt)) {
      return cached.data
    }
    const result = await scanAppRelated(app)
    relatedCache.set(key, { data: result.items, cachedAt: Date.now() })
    return result.items
  }

  const removeAppFromCache = (app: Pick<AppInfo, "path" | "bundle_id">) => {
    if (appsCache) {
      appsCache.data = appsCache.data.filter((a) => a.path !== app.path)
      appsCache.cachedAt = Date.now()
    }
    relatedCache.delete(relatedCacheKey(app))
  }

  const invalidateAll = () => {
    appsCache = null
    relatedCache.clear()
    uiState = null
  }

  const getAppsCachedAt = () => appsCache?.cachedAt ?? null

  const isAppsCacheFresh = () => {
    if (!appsCache) return false
    return isFresh(appsCache.cachedAt)
  }

  const saveUiState = (state: { keyword: string; selectedAppPath: string | null }) => {
    uiState = { ...state, cachedAt: Date.now() }
  }

  const getUiState = (): UninstallUiState | null => {
    if (!uiState || !isFresh(uiState.cachedAt)) {
      uiState = null
      return null
    }
    return uiState
  }

  return {
    loadApps,
    loadRelated,
    removeAppFromCache,
    invalidateAll,
    getAppsCachedAt,
    isAppsCacheFresh,
    saveUiState,
    getUiState,
  }
}

export function formatCacheAge(cachedAt: number | null): string | null {
  if (!cachedAt) return null
  const minutes = Math.floor((Date.now() - cachedAt) / 60_000)
  if (minutes < 1) return "刚刚更新"
  if (minutes < 60) return `${minutes} 分钟前更新`
  return "超过 1 小时"
}
