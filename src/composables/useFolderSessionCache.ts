import type { FolderBrowseSummary } from "@/types/cleaner"
import type { FolderTreeNode } from "@/composables/useFolderTree"

const CACHE_TTL_MS = 60 * 60 * 1000

export type FolderBrowseItem = import("@/types/cleaner").BrowseItem & { selected?: boolean }
export type FolderSortKey = "size" | "time" | "name" | "type"

export interface FolderPageState {
  path: string
  treeNodes: FolderTreeNode[]
  summary: FolderBrowseSummary | null
  scannedOnce: boolean
  sortBy: FolderSortKey
  sortAsc: boolean
  highlightPath: string | null
  scrollTop: number
  cachedAt: number
}

const pageCache = new Map<string, FolderPageState>()

function isFresh(cachedAt: number) {
  return Date.now() - cachedAt < CACHE_TTL_MS
}

export function useFolderSessionCache() {
  const savePageState = (state: Omit<FolderPageState, "cachedAt">) => {
    pageCache.set(state.path, { ...state, cachedAt: Date.now() })
  }

  const getPageState = (path: string): FolderPageState | null => {
    const cached = pageCache.get(path)
    if (!cached || !isFresh(cached.cachedAt)) {
      if (cached) pageCache.delete(path)
      return null
    }
    return cached
  }

  const clearPageCache = () => {
    pageCache.clear()
  }

  const invalidatePath = (path: string) => {
    pageCache.delete(path)
  }

  return {
    savePageState,
    getPageState,
    clearPageCache,
    invalidatePath,
  }
}
