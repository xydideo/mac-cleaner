<template>
  <div class="custom-folder">
    <div
      class="folder-toolbar glass-card"
      :class="{
        'is-collapsed': isHeaderCollapsed,
        'is-dragover': isDragover,
        'has-folder': !!targetFolder,
        'is-empty': !targetFolder,
      }"
      @dragover.prevent="onDragOver"
      @dragleave="isDragover = false"
      @drop.prevent="handleDrop"
      @click="onToolbarClick"
    >
      <div v-if="!targetFolder" class="toolbar-empty">
        <span class="toolbar-empty-icon">📂</span>
        <div class="toolbar-empty-text">
          <p class="toolbar-empty-title">拖入文件夹到此处</p>
          <p class="toolbar-empty-sub">或点击选择目录，树形展开与清理</p>
        </div>
      </div>

      <template v-else>
        <div class="toolbar-main">
          <span class="toolbar-icon">📁</span>
          <div class="toolbar-info">
            <span class="toolbar-name">{{ folderName }}</span>
            <span class="toolbar-path" :title="toolbarPath">{{ toolbarPath }}</span>
          </div>
          <div class="toolbar-actions" @click.stop>
            <button
              v-if="canGoBack"
              class="pill-btn pill-btn--compact"
              @click="goBack"
            >
              返回
            </button>
            <button class="pill-btn pill-btn--compact" @click="pickFolder">重新选择</button>
            <button class="pill-btn pill-btn--compact" @click="openInFinder">访达</button>
            <button class="pill-btn pill-btn--compact" @click="clearFolder">清空</button>
          </div>
        </div>
      </template>
    </div>

    <div
      class="folder-body"
      :class="{ 'is-dragover': isDragover && !targetFolder }"
      @dragover.prevent="onBodyDragOver"
      @dragleave="onBodyDragLeave"
      @drop.prevent="handleDrop"
    >
      <div
        v-if="!targetFolder && !scanning"
        class="shortcuts-panel glass-card"
      >
        <p class="shortcuts-title">快捷目录</p>
        <p class="shortcuts-sub">点击下方目录快速开始，或拖入文件夹到此处</p>
        <div v-if="displayShortcuts.length" class="shortcuts-grid">
          <button
            v-for="item in displayShortcuts"
            :key="item.id"
            type="button"
            class="shortcut-card"
            :class="{ 'is-action': !!item.action }"
            @click="onShortcutClick(item)"
          >
            <span class="shortcut-icon">{{ shortcutIcon(item.id) }}</span>
            <span class="shortcut-label">{{ item.label }}</span>
            <span v-if="item.path" class="shortcut-path" :title="item.path">{{ item.path }}</span>
            <span class="shortcut-desc">{{ item.description }}</span>
          </button>
        </div>
        <p v-else class="shortcuts-empty">正在加载快捷目录…</p>
      </div>

      <div v-if="scanning" class="scan-panel glass-card panel-loading">
        <el-icon class="is-loading" :size="32"><Loading /></el-icon>
        <p class="scan-status">{{ scanPaused ? "扫描已暂停" : "正在读取当前目录…" }}</p>
        <ScanControlButtons :paused="scanPaused" @pause="pauseScan" @resume="resumeScan" />
      </div>

      <div
        v-else-if="treeNodes.length"
        class="result-panel glass-card"
        :class="{ 'is-busy': cleaning }"
      >
        <div class="result-header">
          <div class="result-header-left">
            <div class="result-head-row">
              <span class="summary-size">{{ summarySizeLabel }}</span>
              <div class="list-controls">
                <span class="control-label">排序</span>
                <button
                  v-for="opt in sortOptions"
                  :key="opt.key"
                  class="pill-btn pill-btn--compact"
                  :class="{ 'is-active': sortBy === opt.key }"
                  @click="setSortBy(opt.key)"
                >
                  {{ opt.label }}{{ sortBy === opt.key ? (sortAsc ? " ↑" : " ↓") : "" }}
                </button>
                <span class="list-controls-divider" aria-hidden="true" />
                <button
                  type="button"
                  class="pill-btn pill-btn--compact"
                  :class="{ 'is-active': showAllPaths }"
                  title="显示/隐藏所有项的完整路径"
                  @click="toggleShowAllPaths"
                >
                  路径
                </button>
              </div>
            </div>
            <span class="summary-label" :title="summaryLabelFull">{{ summaryLabelFull }}</span>
            <ScanControlButtons
              v-if="analyzingCount > 0 || hasExactSizing"
              :paused="scanPaused"
              class="analyze-controls"
              @pause="pauseScan"
              @resume="resumeScan"
            />
          </div>
          <div class="result-actions">
            <button class="action-btn action-btn--ghost" :disabled="cleaning" @click="onRefreshList">
              刷新
            </button>
            <button
              class="action-btn action-btn--primary"
              :disabled="selectedCount === 0 || cleaning"
              @click="handleClean('trash')"
            >
              废纸篓 ({{ selectedCount }})
            </button>
            <button
              class="action-btn action-btn--danger"
              :disabled="selectedCount === 0 || cleaning"
              @click="handleClean('permanent')"
            >
              永久删除
            </button>
          </div>
        </div>

        <div ref="resultListRef" class="result-list" @scroll="onResultScroll">
          <div
            v-for="(row, index) in displayRows"
            :key="row.node.id"
            class="browse-item"
            :class="{
              highlighted: row.node.path === highlightPath,
              'is-directory': row.node.is_directory,
              'is-tree-expanded':
                row.node.is_directory && row.node.expanded && row.node.childrenLoaded,
            }"
          >
            <div v-if="row.node.depth > 0" class="tree-guides" aria-hidden="true">
              <span
                v-for="(continues, level) in row.ancestorLines"
                :key="level"
                class="tree-guide-col"
                :class="{
                  'is-through': level < row.ancestorLines.length - 1 && continues,
                  'is-parent': level === row.ancestorLines.length - 1,
                  'is-parent-last': level === row.ancestorLines.length - 1 && row.isLastSibling,
                }"
              />
            </div>
            <button
              v-if="row.node.is_directory"
              type="button"
              class="tree-toggle"
              :class="{ 'is-expanded': row.node.expanded, 'is-loading': row.node.loading }"
              :disabled="cleaning"
              :aria-label="row.node.expanded ? '折叠' : '展开'"
              @click.stop="toggleExpand(row.node)"
            >
              <el-icon class="tree-toggle-chevron"><CaretRight /></el-icon>
              <el-icon v-if="row.node.loading" class="tree-toggle-loading is-loading" :size="11">
                <Loading />
              </el-icon>
            </button>
            <span v-else class="tree-toggle tree-toggle--placeholder" aria-hidden="true" />
            <el-checkbox
              size="small"
              :model-value="!!row.node.selected"
              :disabled="cleaning || row.node.protected"
              @mousedown.shift.prevent
              @click.stop.prevent="onItemSelectClick(row.node, index, $event)"
            />
            <span v-if="row.node.is_directory" class="item-icon">📁</span>
            <img
              v-else-if="showImageThumb(row.node)"
              :src="imagePreviewSrc(row.node.path)"
              :alt="row.node.name"
              class="item-icon item-icon--thumb"
              loading="lazy"
              decoding="async"
              @error="markThumbFailed(row.node.id)"
            />
            <span v-else class="item-icon">📄</span>
            <div class="item-info" :title="itemDescFull(row.node)">
              <div class="item-title-row">
                <button
                  v-if="row.node.is_directory"
                  class="item-name item-name--link"
                  :title="row.node.name"
                  @click="toggleExpand(row.node)"
                >
                  {{ row.node.name }}
                </button>
                <span v-else class="item-name" :title="row.node.name">{{ row.node.name }}</span>
              </div>
              <button
                v-if="showAllPaths"
                type="button"
                class="item-path-line"
                :title="`${row.node.path}（点击复制）`"
                @click.stop="copyItemPath(row.node.path)"
              >
                {{ row.node.path }}
              </button>
            </div>
            <div class="item-side">
              <div class="item-side-main">
                <span class="item-time">{{ row.node.modified }}</span>
                <span class="item-size-group">
                  <span
                    class="item-size"
                    :class="{
                      'is-pending': !row.node.size_ready || row.node.exactSizing,
                      'is-capped': row.node.size_capped && !row.node.exactSizing,
                    }"
                  >
                    {{
                      isRefreshingPath(row.node.path)
                        ? "统计中…"
                        : row.node.exactSizing
                          ? "统计中…"
                          : formatBrowseSize(row.node)
                    }}
                  </span>
                  <button
                    v-if="row.node.is_directory && row.node.size_capped && !row.node.exactSizing"
                    type="button"
                    class="size-exact-btn"
                    title="完整统计该文件夹真实大小（不受 5G / 20 万文件限制）"
                    @click.stop="computeExactSize(row.node)"
                  >
                    具体
                  </button>
                </span>
                <div class="item-side-actions">
                  <button
                    class="path-action"
                    title="重新计算大小"
                    :disabled="cleaning || isRefreshingPath(row.node.path)"
                    @click.stop="refreshItem(row.node)"
                  >
                    刷新
                  </button>
                  <button
                    class="path-action"
                    title="在目录模块中打开"
                    @click.stop="openItemDetail(row.node)"
                  >
                    详情
                  </button>
                  <button
                    class="path-action"
                    title="在访达中显示"
                    @click.stop="openPathInFinder(row.node.path)"
                  >
                    访达
                  </button>
                  <button
                    class="path-action path-action--danger"
                    :title="row.node.protected ? '系统保护项，不可删除' : '删除'"
                    :disabled="cleaning || row.node.protected"
                    @click.stop="deleteSingleItem(row.node)"
                  >
                    删
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <Transition name="page-fade">
          <div v-if="cleaning" class="cleaning-overlay flex-c-c">
            <el-icon class="is-loading" :size="36"><Loading /></el-icon>
            <p>{{ cleaningText }}</p>
          </div>
        </Transition>
      </div>

      <div v-else-if="targetFolder && scannedOnce && !displayRows.length" class="empty-result glass-card">
        <p>当前目录为空或无可显示项</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: "CustomFolder" })

import { computed, nextTick, onMounted, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { CaretRight, Loading } from "@element-plus/icons-vue"
import { ElMessage, ElMessageBox } from "element-plus"
import { open } from "@tauri-apps/plugin-dialog"
import {
  browseFolder,
  analyzeFolderItems,
  fetchFolderShortcuts,
  type FolderShortcut,
} from "@/api/folder"
import { cleanPaths } from "@/api/cleaner"
import ScanControlButtons from "@/components/ScanControlButtons.vue"
import { useAppSettings } from "@/composables/useAppSettings"
import { useScanControl } from "@/composables/useScanControl"
import {
  useFolderSessionCache,
  type FolderPageState,
  type FolderSortKey,
} from "@/composables/useFolderSessionCache"
import {
  collectAllTreeNodes,
  createTreeNode,
  flattenVisibleTreeRows,
  findTreeNode,
  mergeAnalyzedTreeNodes,
  removeTreeNodes,
  type FolderTreeNode,
} from "@/composables/useFolderTree"
import { revealInFinder } from "@/utils/finder"
import { formatBrowseSize, formatAggregateSize } from "@/utils/formatSize"
import { imagePreviewSrc, isImageFile } from "@/utils/imageFile"
import { notifyCleanResult } from "@/utils/cleanResult"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import type { BrowseItem, FolderBrowseSummary } from "@/types/cleaner"

const SCROLL_COLLAPSE_THRESHOLD = 20

const route = useRoute()
const router = useRouter()
const {
  savePageState,
  getPageState,
  clearPageCache,
  invalidatePath,
} = useFolderSessionCache()
const { settings } = useAppSettings()
const { scanPaused, resetScanBackend, pauseScan, resumeScan } = useScanControl()

const browseOptions = computed(() => ({
  showHidden: settings.value.showHiddenFiles,
}))

const targetFolder = ref<string | null>(null)
const highlightPath = ref<string | null>(null)
const isDragover = ref(false)
const isHeaderCollapsed = ref(false)
const scanning = ref(false)
const scannedOnce = ref(false)
const treeNodes = ref<FolderTreeNode[]>([])
const summary = ref<FolderBrowseSummary | null>(null)
const sortBy = ref<FolderSortKey>("size")
const sortAsc = ref(false)
const cleaning = ref(false)
const cleaningText = ref("")
const resultListRef = ref<HTMLElement | null>(null)
const analyzingCount = ref(0)
const folderShortcuts = ref<FolderShortcut[]>([])
const lastSelectedPath = ref<string | null>(null)
const folderNavStack = ref<string[]>([])
const failedThumbIds = ref(new Set<string>())
const showAllPaths = ref(false)
const refreshingPaths = ref(new Set<string>())
let analyzeSession = 0

const canGoBack = computed(() => folderNavStack.value.length > 0)

const resetFailedThumbs = () => {
  failedThumbIds.value = new Set()
}

const showImageThumb = (node: FolderTreeNode) =>
  !node.is_directory && isImageFile(node.name) && !failedThumbIds.value.has(node.id)

const markThumbFailed = (id: string) => {
  if (failedThumbIds.value.has(id)) return
  const next = new Set(failedThumbIds.value)
  next.add(id)
  failedThumbIds.value = next
}

const shortcutIcon = (id: string) =>
  ({
    home: "🏠",
    downloads: "⬇️",
    documents: "📝",
    library: "📚",
  }[id] ?? "📁")

const displayShortcuts = computed(() => folderShortcuts.value)

onMounted(async () => {
  if (!isTauriRuntime()) return
  try {
    folderShortcuts.value = await fetchFolderShortcuts()
  } catch {
    folderShortcuts.value = []
  }
})

const onShortcutClick = async (item: FolderShortcut) => {
  if (item.path) {
    await openShortcut(item.path)
  }
}

const openShortcut = async (path: string) => {
  await setFolder(path)
}

const sortOptions: { key: FolderSortKey; label: string }[] = [
  { key: "size", label: "大小" },
  { key: "time", label: "时间" },
  { key: "name", label: "名称" },
  { key: "type", label: "类型" },
]

const folderName = computed(() => {
  if (!targetFolder.value) return ""
  return targetFolder.value.split("/").pop() ?? targetFolder.value
})

const toolbarPath = computed(() => targetFolder.value ?? "")

const allLoadedNodes = computed(() => collectAllTreeNodes(treeNodes.value))

const displayRows = computed(() =>
  flattenVisibleTreeRows(treeNodes.value, sortBy.value, sortAsc.value)
)

const selectedCount = computed(() => allLoadedNodes.value.filter((i) => i.selected).length)

const summarySizeLabel = computed(() => {
  const ready = allLoadedNodes.value.filter((i) => i.size_ready)
  const total = ready.reduce((s, i) => s + i.size_bytes, 0)
  const hasCapped = ready.some((i) => i.size_capped)
  return formatAggregateSize(total, { hasCapped })
})

const selectedSizeLabel = computed(() => {
  const selected = allLoadedNodes.value.filter((i) => i.selected)
  const total = selected.reduce((s, i) => s + i.size_bytes, 0)
  const hasCapped = selected.some((i) => i.size_capped)
  return formatAggregateSize(total, { hasCapped })
})

const summaryLabelFull = computed(() => {
  const parts = [
    `已加载 ${summary.value?.folder_count ?? 0} 个文件夹 · ${summary.value?.file_count ?? 0} 个文件`,
    `已选 ${selectedCount.value} 项 · ${selectedSizeLabel.value}`,
  ]
  if (analyzingCount.value > 0) {
    parts.push(scanPaused.value ? "计算已暂停" : `${analyzingCount.value} 项计算中…`)
  } else if (hasExactSizing.value) {
    parts.push(scanPaused.value ? "精确统计已暂停" : "精确统计中…")
  }
  return parts.join(" | ")
})

const hasExactSizing = computed(() =>
  allLoadedNodes.value.some((node) => node.exactSizing)
)

const setItemSelected = (path: string, selected: boolean) => {
  const target = findTreeNode(treeNodes.value, path)
  if (target && !target.protected) {
    target.selected = selected
  }
}

const onItemSelectClick = (
  item: FolderTreeNode,
  index: number,
  event: MouseEvent
) => {
  if (cleaning.value || item.protected) return

  const visible = displayRows.value.map((row) => row.node)

  if (event.shiftKey && lastSelectedPath.value) {
    const anchorIdx = visible.findIndex((i) => i.path === lastSelectedPath.value)
    if (anchorIdx >= 0) {
      const from = Math.min(anchorIdx, index)
      const to = Math.max(anchorIdx, index)
      for (let i = from; i <= to; i++) {
        setItemSelected(visible[i].path, true)
      }
      lastSelectedPath.value = item.path
      window.getSelection()?.removeAllRanges()
      persistCurrentPage()
      return
    }
  }

  setItemSelected(item.path, !item.selected)
  lastSelectedPath.value = item.path
  persistCurrentPage()
}

const FILES_CAPPED_HINT = "（已统计 20万个文件，目录可能更大）"

const itemDescParts = (item: FolderTreeNode) => {
  const { description } = item
  if (description.endsWith(FILES_CAPPED_HINT)) {
    return {
      base: description.slice(0, -FILES_CAPPED_HINT.length),
      capped: FILES_CAPPED_HINT,
    }
  }
  return { base: description, capped: null as string | null }
}

const itemDescFull = (item: FolderTreeNode) => {
  const capped = itemDescParts(item)
  let text = capped.capped ? `${capped.base}${capped.capped}` : item.description
  if (item.is_directory && item.child_count) {
    text += ` · ${item.child_count} 项`
  }
  return text
}

const setSortBy = (key: FolderSortKey) => {
  if (sortBy.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortBy.value = key
    sortAsc.value = key === "name" || key === "type"
  }
  persistCurrentPage()
}

const onResultScroll = (e: Event) => {
  const top = (e.target as HTMLElement).scrollTop
  isHeaderCollapsed.value = top > SCROLL_COLLAPSE_THRESHOLD
}

const resetHeaderCollapse = () => {
  isHeaderCollapsed.value = false
  if (resultListRef.value) {
    resultListRef.value.scrollTop = 0
  }
}

const onDragOver = () => {
  if (!targetFolder.value) {
    isDragover.value = true
  }
}

const onBodyDragOver = () => {
  if (!targetFolder.value) {
    isDragover.value = true
  }
}

const onBodyDragLeave = (e: DragEvent) => {
  const related = e.relatedTarget as Node | null
  const current = e.currentTarget as HTMLElement
  if (related && current.contains(related)) return
  isDragover.value = false
}

const onToolbarClick = () => {
  if (!targetFolder.value) {
    void pickFolder()
  }
}

const recomputeSummary = () => {
  if (!targetFolder.value) return
  const items = allLoadedNodes.value
  summary.value = {
    path: targetFolder.value,
    total_bytes: items.filter((i) => i.size_ready).reduce((s, i) => s + i.size_bytes, 0),
    folder_count: items.filter((i) => i.is_directory).length,
    file_count: items.filter((i) => !i.is_directory).length,
    junk_bytes: items.filter((i) => i.size_ready).reduce((s, i) => s + i.junk_bytes, 0),
    junk_count: items.filter((i) => i.size_ready).reduce((s, i) => s + i.junk_count, 0),
    pending_count: items.filter((i) => i.is_directory && !i.size_ready).length,
  }
}

const mergeAnalyzedItems = (updates: BrowseItem[]) => {
  treeNodes.value = mergeAnalyzedTreeNodes(treeNodes.value, updates)
  recomputeSummary()
  persistCurrentPage()
}

const computeExactSize = async (node: FolderTreeNode) => {
  if (!node.is_directory || cleaning.value || node.exactSizing) return

  node.exactSizing = true
  try {
    const { items } = await analyzeFolderItems([node.path], { exact: true })
    mergeAnalyzedItems(items)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    const target = findTreeNode(treeNodes.value, node.path)
    if (target) target.exactSizing = false
  }
}

const toggleShowAllPaths = () => {
  showAllPaths.value = !showAllPaths.value
}

const copyItemPath = async (path: string) => {
  try {
    await navigator.clipboard.writeText(path)
    ElMessage.success("路径已复制")
  } catch {
    ElMessage.error("复制失败")
  }
}

const isRefreshingPath = (path: string) => refreshingPaths.value.has(path)

const refreshItem = async (node: FolderTreeNode) => {
  if (cleaning.value || isRefreshingPath(node.path)) return

  const next = new Set(refreshingPaths.value)
  next.add(node.path)
  refreshingPaths.value = next

  try {
    const { items } = await analyzeFolderItems([node.path])
    if (!items.length) {
      ElMessage.warning("无法刷新该项")
      return
    }
    mergeAnalyzedItems(items)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    const done = new Set(refreshingPaths.value)
    done.delete(node.path)
    refreshingPaths.value = done
  }
}

const toggleExpand = async (node: FolderTreeNode) => {
  if (!node.is_directory || cleaning.value) return

  if (node.expanded) {
    node.expanded = false
    persistCurrentPage()
    return
  }

  if (node.childrenLoaded) {
    node.expanded = true
    persistCurrentPage()
    return
  }

  node.expanded = true
  node.loading = true
  const session = analyzeSession
  try {
    const result = await browseFolder(node.path, browseOptions.value)
    if (session !== analyzeSession) return
    node.children = result.items.map((item, index) =>
      createTreeNode(item, node.depth + 1, index)
    )
    node.childrenLoaded = true
    persistCurrentPage()
    void analyzePendingFolders(session)
  } catch (e) {
    node.expanded = false
    ElMessage.error(String(e))
  } finally {
    if (session === analyzeSession) {
      node.loading = false
    }
  }
}

const analyzePendingFolders = async (session: number) => {
  const pending = allLoadedNodes.value
    .filter((i) => i.is_directory && !i.size_ready)
    .map((i) => i.path)
  if (!pending.length || session !== analyzeSession) return

  analyzingCount.value = pending.length
  try {
    const { items } = await analyzeFolderItems(pending)
    if (session !== analyzeSession) return
    mergeAnalyzedItems(items)
  } catch (e) {
    if (session === analyzeSession) {
      ElMessage.error(String(e))
    }
  } finally {
    if (session === analyzeSession) {
      analyzingCount.value = allLoadedNodes.value.filter(
        (i) => i.is_directory && !i.size_ready
      ).length
    }
  }
}

const persistCurrentPage = () => {
  if (!targetFolder.value) return
  savePageState({
    path: targetFolder.value,
    treeNodes: JSON.parse(JSON.stringify(treeNodes.value)) as FolderTreeNode[],
    summary: summary.value ? { ...summary.value } : null,
    scannedOnce: scannedOnce.value,
    sortBy: sortBy.value,
    sortAsc: sortAsc.value,
    highlightPath: highlightPath.value,
    scrollTop: resultListRef.value?.scrollTop ?? 0,
  })
}

const restoreTreeNode = (node: FolderTreeNode): FolderTreeNode => ({
  ...node,
  size_ready: node.size_ready ?? node.is_directory === false,
  size_capped: node.size_capped ?? false,
  protected: node.protected ?? false,
  listOrder: node.listOrder ?? 0,
  loading: false,
  children: (node.children ?? []).map(restoreTreeNode),
  exactSizing: false,
})

const restorePageState = async (state: FolderPageState) => {
  targetFolder.value = state.path
  treeNodes.value = state.treeNodes.map(restoreTreeNode)
  summary.value = state.summary
  scannedOnce.value = state.scannedOnce
  sortBy.value = state.sortBy
  sortAsc.value = state.sortAsc
  highlightPath.value = state.highlightPath
  resetHeaderCollapse()
  await nextTick()
  if (resultListRef.value) {
    resultListRef.value.scrollTop = state.scrollTop
  }
  const session = ++analyzeSession
  void analyzePendingFolders(session)
}

const loadPath = async (
  path: string,
  opts: { useCache?: boolean; forceScan?: boolean } = {}
) => {
  const { useCache = true, forceScan = false } = opts

  if (useCache && !forceScan) {
    const cached = getPageState(path)
    if (cached) {
      await restorePageState(cached)
      return
    }
  }

  targetFolder.value = path
  scannedOnce.value = false
  treeNodes.value = []
  summary.value = null
  lastSelectedPath.value = null
  resetFailedThumbs()
  analyzeSession += 1
  analyzingCount.value = 0
  resetHeaderCollapse()
  await runBrowse()
}

const enterFolder = async (path: string, highlight: string | null = null) => {
  highlightPath.value = highlight
  await loadPath(path, { useCache: true })
}

const setFolder = async (path: string) => {
  folderNavStack.value = []
  await enterFolder(path)
  router.replace({ name: "CustomFolder", query: { path } })
}

const goBack = async () => {
  const prev = folderNavStack.value.pop()
  if (!prev) return

  highlightPath.value = null
  lastSelectedPath.value = null
  await loadPath(prev, { useCache: true })
  router.replace({ name: "CustomFolder", query: { path: prev } })
}

const clearFolder = () => {
  targetFolder.value = null
  treeNodes.value = []
  summary.value = null
  scannedOnce.value = false
  highlightPath.value = null
  lastSelectedPath.value = null
  folderNavStack.value = []
  resetFailedThumbs()
  resetHeaderCollapse()
  clearPageCache()
  router.replace({ name: "CustomFolder", query: {} })
}

const pickFolder = async () => {
  if (!isTauriRuntime()) return
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    await setFolder(selected as string)
  }
}

const handleDrop = async (e: DragEvent) => {
  isDragover.value = false
  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    const path = (files[0] as File & { path?: string }).path
    if (path) {
      await setFolder(path)
    }
  }
}

const runBrowse = async () => {
  if (!targetFolder.value) return
  const session = ++analyzeSession
  scanning.value = true
  analyzingCount.value = 0
  resetHeaderCollapse()
  await resetScanBackend()
  try {
    const result = await browseFolder(targetFolder.value, browseOptions.value)
    if (session !== analyzeSession) return
    treeNodes.value = result.items.map((item, index) => createTreeNode(item, 0, index))
    summary.value = result.summary
    scannedOnce.value = true
    persistCurrentPage()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    if (session === analyzeSession) {
      scanning.value = false
    }
  }
  void analyzePendingFolders(session)
}

watch(
  () => settings.value.showHiddenFiles,
  () => {
    if (targetFolder.value && scannedOnce.value) {
      void runBrowse()
    }
  }
)

const onRefreshList = async () => {
  await runBrowse()
}

const openInFinder = () => {
  if (targetFolder.value) openPathInFinder(targetFolder.value)
}

const openPathInFinder = async (path: string) => {
  try {
    await revealInFinder(path)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const openItemDetail = async (item: FolderTreeNode) => {
  const focusPath = item.is_directory
    ? item.path
    : item.path.slice(0, item.path.lastIndexOf("/"))

  if (!focusPath) return

  if (targetFolder.value && targetFolder.value !== focusPath) {
    folderNavStack.value.push(targetFolder.value)
  }

  lastSelectedPath.value = null
  const highlight = item.is_directory ? null : item.path
  await enterFolder(focusPath, highlight)
  router.replace({
    name: "CustomFolder",
    query: highlight ? { path: focusPath, highlight } : { path: focusPath },
  })
}

const formatSelectedSizeLabel = (items: FolderTreeNode[]) => {
  const total = items.reduce((s, i) => s + i.size_bytes, 0)
  const hasCapped = items.some((i) => i.size_capped)
  return formatAggregateSize(total, { hasCapped })
}

const confirmDeleteItems = async (
  items: FolderTreeNode[],
  mode: "trash" | "permanent"
): Promise<boolean> => {
  if (items.length === 0) return false

  const hasFolder = items.some((i) => i.is_directory)
  const sizeLabel = formatSelectedSizeLabel(items)
  const names = items.slice(0, 3).map((i) => i.name)
  const nameHint =
    items.length > 3 ? `${names.join("、")} 等 ${items.length} 项` : names.join("、")

  if (hasFolder) {
    await ElMessageBox.confirm(
      `即将删除整个文件夹：${nameHint}（${sizeLabel}）。文件夹内所有内容将被一并删除。`,
      "删除文件夹确认",
      { confirmButtonText: "继续", cancelButtonText: "取消", type: "warning" }
    )
    await ElMessageBox.confirm(
      `请再次确认：是否${mode === "trash" ? "移到废纸篓" : "永久删除"}上述文件夹？此操作不可撤销文件夹层级。`,
      "二次确认",
      {
        confirmButtonText: mode === "trash" ? "移到废纸篓" : "永久删除",
        cancelButtonText: "取消",
        type: "error",
        confirmButtonClass: mode === "permanent" ? "el-button--danger" : undefined,
      }
    )
  } else {
    await ElMessageBox.confirm(
      `即将删除 ${items.length} 项（${sizeLabel}）：${nameHint}。删除前请自行确认。`,
      "确认删除",
      { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" }
    )
  }

  if (mode === "permanent") {
    await ElMessageBox.confirm("永久删除不可恢复，请最终确认。", "最终确认", {
      confirmButtonText: "永久删除",
      cancelButtonText: "取消",
      type: "error",
      confirmButtonClass: "el-button--danger",
    })
  }

  return true
}

const executeDelete = async (items: FolderTreeNode[], mode: "trash" | "permanent") => {
  const paths = items.map((i) => i.path)
  const sizes = items.map((i) => i.size_bytes)

  cleaning.value = true
  cleaningText.value = mode === "trash" ? "正在移到废纸篓..." : "正在永久删除..."

  try {
    const result = await cleanPaths(paths, mode, sizes)
    await notifyCleanResult(result, mode === "trash" ? "移到废纸篓" : "永久删除")

    const removed = new Set(result.success)
    treeNodes.value = removeTreeNodes(treeNodes.value, removed)

    if (targetFolder.value) {
      invalidatePath(targetFolder.value)
    }

    recomputeSummary()
    persistCurrentPage()
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    cleaning.value = false
    cleaningText.value = ""
  }
}

const deleteSingleItem = async (item: FolderTreeNode) => {
  try {
    const ok = await confirmDeleteItems([item], "trash")
    if (!ok) return
    await executeDelete([item], "trash")
  } catch {
    // 用户取消
  }
}

const handleClean = async (mode: "trash" | "permanent") => {
  const selected = allLoadedNodes.value.filter((i) => i.selected)
  if (selected.length === 0) return

  try {
    const ok = await confirmDeleteItems(selected, mode)
    if (!ok) return
    await executeDelete(selected, mode)
  } catch {
    // 用户取消
  }
}

watch(
  () => route.query.path,
  async (path) => {
    if (typeof path !== "string" || !path.trim()) return
    const normalized = path.trim()
    if (normalized === targetFolder.value) return

    const highlight = typeof route.query.highlight === "string" ? route.query.highlight : null
    await enterFolder(normalized, highlight)
  },
  { immediate: true }
)
</script>

<style scoped lang="scss">
.custom-folder {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding-top: 8px;
  height: calc(100vh - #{$header-height} - 28px);
  min-height: 0;
}

.folder-toolbar {
  flex-shrink: 0;
  overflow: hidden;
  border: 2px dashed rgba(124, 58, 237, 0.25);
  cursor: pointer;
  transition:
    border-color 0.28s ease,
    box-shadow 0.28s ease,
    padding 0.32s cubic-bezier(0.22, 1, 0.36, 1),
    min-height 0.32s cubic-bezier(0.22, 1, 0.36, 1);

  &.is-empty {
    min-height: 160px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;

    &:hover,
    &.is-dragover {
      border-color: rgba(124, 58, 237, 0.6);
      box-shadow: $shadow-glow;
    }
  }

  &.has-folder:not(.is-empty) {
    cursor: default;
    border-style: solid;
    border-color: rgba(16, 185, 129, 0.35);
    padding: 18px 20px;
    min-height: 88px;
  }

  &.has-folder.is-collapsed {
    padding: 8px 14px;
    min-height: 48px;
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 12px;
    border-color: rgba(255, 255, 255, 0.1);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
  }
}

.toolbar-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
}

.toolbar-empty-icon {
  font-size: 44px;
  line-height: 1;
}

.toolbar-empty-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: $text-primary;
}

.toolbar-empty-sub {
  margin: 0;
  font-size: 13px;
  color: $text-muted;
}

.toolbar-main {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;

  .is-collapsed & {
    flex: 1;
    gap: 8px;
  }
}

.toolbar-icon {
  font-size: 36px;
  line-height: 1;
  flex-shrink: 0;

  .is-collapsed & {
    font-size: 20px;
  }
}

.toolbar-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;

  .is-collapsed & {
    flex-direction: row;
    align-items: center;
    gap: 8px;
  }
}

.toolbar-name {
  font-size: 16px;
  font-weight: 600;
  color: $text-primary;
  white-space: nowrap;

  .is-collapsed & {
    font-size: 13px;
  }
}

.toolbar-path {
  font-size: 12px;
  color: $text-muted;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 520px;

  .is-collapsed & {
    font-size: 11px;
    max-width: 280px;
    opacity: 0.75;

    &::before {
      content: "· ";
      opacity: 0.5;
    }
  }
}

.toolbar-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex-shrink: 0;
}

.result-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 18px;
  border-bottom: 1px solid $border-subtle;
  flex-shrink: 0;
  min-width: 0;
}

.result-header-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.result-head-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px 12px;
  min-width: 0;
}

.list-controls {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
}

.list-controls-divider {
  width: 1px;
  height: 14px;
  background: $border-subtle;
  flex-shrink: 0;
}

.control-label {
  font-size: 12px;
  color: $text-muted;
  flex-shrink: 0;
}

.summary-size {
  font-size: 20px;
  font-weight: 700;
  color: $color-green;
  flex-shrink: 0;
  line-height: 1.2;
}

.summary-label {
  font-size: 12px;
  color: $text-muted;
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: default;
}

.analyze-controls {
  flex-shrink: 0;
}

.pill-btn--compact {
  padding: 4px 10px;
  font-size: 11px;

  &.is-active {
    background: rgba(124, 58, 237, 0.25);
    border-color: rgba(124, 58, 237, 0.5);
    color: $text-primary;
  }
}

.result-actions {
  display: flex;
  flex-wrap: nowrap;
  gap: 8px;
  flex-shrink: 0;
  padding-top: 2px;
}

.folder-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;

  &.is-dragover .shortcuts-panel {
    border-color: rgba(124, 58, 237, 0.55);
    box-shadow: $shadow-glow;
  }
}

.shortcuts-panel {
  flex: 1;
  min-height: 0;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  border: 2px dashed rgba(124, 58, 237, 0.2);
  transition:
    border-color 0.28s ease,
    box-shadow 0.28s ease;
}

.shortcuts-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: $text-primary;
}

.shortcuts-sub {
  margin: 0;
  font-size: 13px;
  color: $text-muted;
}

.shortcuts-empty {
  margin: 0;
  font-size: 13px;
  color: $text-muted;
}

.shortcuts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
  margin-top: 4px;
}

.shortcut-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 16px;
  border-radius: $radius-button;
  border: 1px solid $border-subtle;
  background: rgba(255, 255, 255, 0.04);
  cursor: pointer;
  text-align: left;
  transition:
    border-color 0.2s ease,
    background 0.2s ease,
    transform 0.2s ease;

  &:hover {
    border-color: rgba(124, 58, 237, 0.45);
    background: rgba(124, 58, 237, 0.1);
    transform: translateY(-1px);
  }

  &.is-action {
    border-color: rgba(16, 185, 129, 0.25);
    background: rgba(16, 185, 129, 0.06);

    &:hover {
      border-color: rgba(16, 185, 129, 0.45);
      background: rgba(16, 185, 129, 0.1);
    }
  }
}

.shortcut-icon {
  font-size: 24px;
  line-height: 1;
}

.shortcut-label {
  font-size: 15px;
  font-weight: 600;
  color: $text-primary;
}

.shortcut-path {
  font-size: 11px;
  color: $text-muted;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.shortcut-desc {
  font-size: 12px;
  color: $text-secondary;
  line-height: 1.45;
}

.shortcut-settings-link {
  margin-top: 6px;
  padding: 0;
  border: none;
  background: none;
  color: $color-blue;
  font-size: 11px;
  cursor: pointer;

  &:hover {
    text-decoration: underline;
  }
}

.scan-panel,
.empty-result {
  padding: 24px;
  text-align: center;
  color: $text-secondary;
}

.scan-progress-lines {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  width: 100%;
  max-width: 560px;
}

.scan-status {
  margin: 0;
  font-size: 14px;
  color: $text-secondary;
}

.scan-path {
  margin: 0;
  font-size: 12px;
  color: $text-muted;

  &--current {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &--stats {
    font-variant-numeric: tabular-nums;
  }
}

.panel-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.result-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.result-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px;
  user-select: none;
}

.browse-item {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 6px;
  min-height: 30px;
  position: relative;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);

  &.highlighted {
    background: rgba(124, 58, 237, 0.12);
    border-radius: $radius-button;
  }

  &.is-directory .item-icon {
    filter: none;
  }
}

$tree-indent: 14px;

.tree-guides {
  display: flex;
  flex-shrink: 0;
  align-self: stretch;
}

.tree-guide-col {
  position: relative;
  width: $tree-indent;
  flex-shrink: 0;
  align-self: stretch;

  &.is-through::before {
    content: "";
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 1px;
    transform: translateX(-50%);
    background: rgba(255, 255, 255, 0.1);
  }

  &.is-parent::before {
    content: "";
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 1px;
    transform: translateX(-50%);
    background: rgba(124, 58, 237, 0.28);
  }

  &.is-parent.is-parent-last::before {
    bottom: 50%;
  }

  &.is-parent::after {
    content: "";
    position: absolute;
    left: 50%;
    right: 0;
    top: 50%;
    height: 1px;
    transform: translateY(-50%);
    background: rgba(124, 58, 237, 0.28);
  }
}

.tree-toggle {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  margin-top: 0;
  padding: 0;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.04);
  color: $text-secondary;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1;
  transition:
    background 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    box-shadow 0.2s ease;

  &:hover:not(:disabled) {
    background: rgba(124, 58, 237, 0.16);
    border-color: rgba(124, 58, 237, 0.38);
    color: #ddd6fe;
    box-shadow: 0 0 0 2px rgba(124, 58, 237, 0.08);
  }

  &:active:not(:disabled) {
    transform: scale(0.94);
  }

  &.is-expanded {
    background: rgba(124, 58, 237, 0.12);
    border-color: rgba(124, 58, 237, 0.28);
    color: #c4b5fd;
  }

  &.is-loading .tree-toggle-chevron {
    opacity: 0.35;
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  &--placeholder {
    cursor: default;
    pointer-events: none;
    visibility: hidden;
  }
}

.tree-toggle-chevron {
  font-size: 10px;
  transition: transform 0.22s cubic-bezier(0.22, 1, 0.36, 1);
}

.tree-toggle.is-expanded .tree-toggle-chevron {
  transform: rotate(90deg);
}

.browse-item.is-tree-expanded .tree-toggle::after {
  content: "";
  position: absolute;
  left: 50%;
  top: 100%;
  width: 1px;
  height: 6px;
  transform: translateX(-50%);
  background: rgba(124, 58, 237, 0.28);
  pointer-events: none;
}

.tree-toggle-loading {
  position: absolute;
  inset: 0;
  margin: auto;
  color: $color-purple;
}

.item-icon {
  font-size: 16px;
  line-height: 1;
  flex-shrink: 0;

  &--thumb {
    width: 28px;
    height: 28px;
    object-fit: cover;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title-row {
  display: flex;
  align-items: center;
  min-width: 0;
}

.item-name {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 600;
  color: $text-primary;
  background: none;
  border: none;
  padding: 0;
  text-align: left;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.3;

  &--link {
    cursor: pointer;
    color: $color-blue;

    &:hover {
      text-decoration: underline;
    }
  }
}

.item-path-line {
  display: block;
  width: 100%;
  margin-top: 2px;
  padding: 0;
  border: none;
  background: none;
  text-align: left;
  font-size: 10px;
  line-height: 1.35;
  color: $text-muted;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  &:hover {
    color: $color-purple;
    text-decoration: underline;
  }
}

.item-side {
  flex-shrink: 0;
  margin-left: 8px;
}

.item-side-main {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  white-space: nowrap;
}

.item-time {
  font-size: 10px;
  color: $text-muted;
  font-variant-numeric: tabular-nums;
}

.item-size-group {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.item-size {
  font-size: 11px;
  font-weight: 600;
  color: $text-secondary;
  font-variant-numeric: tabular-nums;
  min-width: 72px;
  text-align: right;
  display: inline-block;

  &.is-pending {
    color: $text-muted;
    font-weight: 500;
    font-size: 10px;
  }

  &.is-capped {
    color: $color-amber;
  }
}

.size-exact-btn {
  flex-shrink: 0;
  padding: 0 4px;
  border: none;
  background: transparent;
  color: $color-amber;
  font-size: 10px;
  line-height: 1.4;
  cursor: pointer;
  white-space: nowrap;
  opacity: 0.85;

  &:hover {
    opacity: 1;
    text-decoration: underline;
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

.item-side-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 2px;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
}

.path-action {
  flex-shrink: 0;
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid $border-subtle;
  background: rgba(255, 255, 255, 0.04);
  color: $text-secondary;
  font-size: 10px;
  line-height: 1.4;
  cursor: pointer;
  white-space: nowrap;

  &:hover:not(:disabled) {
    background: $bg-card-hover;
    color: $text-primary;
  }

  &--danger:hover:not(:disabled) {
    border-color: rgba(239, 68, 68, 0.4);
    color: #f87171;
  }

  &.is-active {
    border-color: rgba(124, 58, 237, 0.35);
    color: $color-purple;
    background: rgba(124, 58, 237, 0.12);
  }

  &:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
}

.cleaning-overlay {
  position: absolute;
  inset: 0;
  z-index: 10;
  flex-direction: column;
  gap: 12px;
  background: rgba(13, 13, 18, 0.85);
  backdrop-filter: blur(6px);
  border-radius: $radius-card;
  color: $text-primary;
}

.flex-c-c {
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
