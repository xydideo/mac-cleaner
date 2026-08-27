<template>
  <div class="large-file-scan">
    <div v-if="currentItems.length && !scanning" class="warn-banner glass-card">
      <span class="warn-icon">⚠️</span>
      <div class="warn-text">
        <p class="warn-title">删除前请谨慎确认</p>
        <p class="warn-desc">
          扫描结果可能包含仍在使用的文件或系统目录，请自行判断后再操作。
        </p>
      </div>
    </div>

    <div v-if="scanning" class="scan-panel glass-card panel-loading">
      <ScanKindSegmented
        class="scan-panel-tabs"
        :model-value="scanningKind"
        disabled
      />
      <el-icon class="is-loading" :size="36"><Loading /></el-icon>
      <div class="scan-progress-lines">
        <p class="scan-status">
          {{ scanPaused ? "扫描已暂停" : scanStatusText }}
        </p>
        <p
          v-if="scanProgressCurrentPath"
          class="scan-path scan-path--current"
          :title="scanProgressCurrentPath"
        >
          {{ scanProgressCurrentPath }}
        </p>
        <p class="scan-path scan-path--stats">
          （已扫描 <span class="scan-stat-num">{{ scanProgressFilesScannedLabel }}</span> 个文件，找到
          <span class="scan-stat-num scan-stat-num--found">{{ scanProgressItemsFound }}</span> 项）
        </p>
        <ScanControlButtons :paused="scanPaused" @pause="pauseScan" @resume="resumeScan" />
        <div class="scan-actions">
          <button
            class="action-btn action-btn--ghost"
            :disabled="stopping"
            @click="stopScan"
          >
            {{ stopping ? "正在结束…" : "结束扫描" }}
          </button>
        </div>
      </div>
    </div>

    <div
      v-else-if="currentItems.length"
      class="result-panel glass-card"
      :class="{ 'is-busy': cleaning }"
    >
      <div class="result-header">
        <div class="result-summary" :title="currentScanMetaLabel">
          <p class="result-summary-line">
            <span class="summary-size">{{ formatSize(selectedBytes) }}</span>
            <span class="summary-divider">|</span>
            <span class="summary-detail">
              已选 / 共 {{ formatSize(currentTotalBytes) }} · {{ currentItems.length }} 项
            </span>
          </p>
        </div>
        <div class="result-actions">
          <ScanKindSegmented
            :model-value="activeTab"
            :disabled="scanning"
            @change="onTabChange"
          />
          <button class="action-btn action-btn--ghost" :disabled="cleaning" @click="runScan()">
            重新扫描
          </button>
          <button
            class="action-btn action-btn--primary"
            :disabled="selectedCount === 0 || cleaning"
            @click="handleCleanTrash"
          >
            移到废纸篓 ({{ selectedCount }})
          </button>
        </div>
      </div>

      <div class="result-list">
        <div
          v-for="item in sortedItems"
          :key="item.id"
          class="scan-item"
          :class="{ 'is-protected': item.protected }"
        >
          <el-checkbox
            v-model="item.selected"
            :disabled="cleaning || item.protected"
          />
          <span class="item-icon">{{ item.is_directory ? "📁" : "📄" }}</span>
          <div class="item-main">
            <div class="item-title-row">
              <span class="item-name" :title="item.name">{{ item.name }}</span>
              <span
                v-if="item.app_label"
                class="app-tag"
                :class="{
                  'is-uninstalled': item.app_installed === false,
                  'is-system': item.app_label === '系统' || item.protected,
                }"
                :title="item.bundle_id ? `Bundle ID: ${item.bundle_id}` : item.app_label"
              >
                {{ item.app_label }}
              </span>
            </div>
            <button
              type="button"
              class="item-path"
              :title="`${item.path}（点击复制）`"
              @click="copyItemPath(item.path)"
            >
              {{ item.path }}
            </button>
          </div>
          <div class="item-meta">
            <span class="item-time">{{ item.modified }}</span>
            <span class="item-size">{{ formatSize(item.size_bytes) }}</span>
          </div>
          <div class="item-actions">
            <button
              class="path-action"
              title="在目录模块中打开"
              @click="openItemDetail(item)"
            >
              详情
            </button>
            <button class="path-action" @click="openInFinder(item.path)">访达</button>
            <button
              class="path-action path-action--danger"
              :title="item.protected ? '系统保护项，不可删除' : '删除'"
              :disabled="cleaning || item.protected"
              @click="deleteSingle(item)"
            >
              删
            </button>
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

    <div v-else-if="currentTabScannedOnce" class="empty-panel glass-card">
      <ScanKindSegmented
        class="empty-panel-tabs"
        :model-value="activeTab"
        @change="onTabChange"
      />
      <p class="empty-title">{{ emptyTitle }}</p>
      <p class="empty-sub">{{ emptySubtitle }}</p>
      <p class="scan-meta">{{ currentScanMetaLabel }}</p>
      <div class="empty-actions">
        <button class="action-btn action-btn--ghost" @click="runScan()">重新扫描</button>
        <button class="action-btn action-btn--primary" @click="goSettings">调整阈值</button>
      </div>
    </div>

    <div v-else class="idle-panel glass-card">
      <ScanKindSegmented
        class="idle-panel-tabs"
        :model-value="activeTab"
        @change="onTabChange"
      />
      <span class="idle-icon">🔍</span>
      <p class="idle-title">查找占用空间的大文件与文件夹</p>
      <p class="idle-sub">
        默认先扫描大文件夹；切换至「大文件」将单独全盘扫描，<br />通常需要 10 分钟以上，甚至可能更久
      </p>
      <p class="idle-scope">扫描范围：主目录 / 应用程序 / 系统 / 磁盘卷</p>
      <div class="idle-threshold-row">
        <span class="idle-threshold-label">{{ currentThresholdLabel }}</span>
        <button class="pill-btn pill-btn--compact" @click="goSettings">阈值配置</button>
      </div>
      <div class="idle-actions">
        <button class="action-btn action-btn--primary" @click="startInitialScan">开始扫描</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: "LargeFileScan" })

import { computed, onUnmounted, ref } from "vue"
import { useRouter } from "vue-router"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import { Loading } from "@element-plus/icons-vue"
import { ElMessage, ElMessageBox } from "element-plus"
import ScanControlButtons from "@/components/ScanControlButtons.vue"
import ScanKindSegmented from "@/components/ScanKindSegmented.vue"
import { scanLargeFiles, type LargeScanKind } from "@/api/folder"
import {
  useLargeFileScanCache,
  type LargeScanItem,
  type LargeTabScanState,
} from "@/composables/useLargeFileScanCache"
import { useAppSettings } from "@/composables/useAppSettings"
import { useCleanAction } from "@/composables/useCleanAction"
import { useScanControl } from "@/composables/useScanControl"
import { revealInFinder } from "@/utils/finder"
import { navigateToFolderScan } from "@/utils/folderNavigation"
import { formatSize } from "@/utils/formatSize"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import type { CleanResult } from "@/types/cleaner"

const router = useRouter()
const { largeFileBytes, largeFolderBytes, formatThresholdMB, settings } = useAppSettings()
const { activeTab, tabState } = useLargeFileScanCache()
const { scanPaused, resetScanControlState, resetScanBackend, pauseScan, resumeScan, cancelScan } =
  useScanControl()
const { cleaning, cleaningText, cleanWithConfirm } = useCleanAction()

const scanning = ref(false)
const scanningKind = ref<LargeScanKind>("folders")
const stopping = ref(false)
const scanStoppedByUser = ref(false)
const scanProgressCurrentPath = ref("")
const scanProgressFilesScanned = ref(0)
const scanProgressItemsFound = ref(0)
let progressUnlisten: UnlistenFn | null = null

const currentState = computed(() => tabState(activeTab.value).value)
const currentItems = computed(() => currentState.value.items)
const currentTotalBytes = computed(() => currentState.value.totalBytes)
const currentTabScannedOnce = computed(() => currentState.value.scannedOnce)

const sortedItems = computed(() =>
  [...currentItems.value].sort((a, b) => b.size_bytes - a.size_bytes || a.path.localeCompare(b.path))
)
const selectedCount = computed(() =>
  currentItems.value.filter((i) => i.selected && !i.protected).length
)
const selectedBytes = computed(() =>
  currentItems.value
    .filter((i) => i.selected && !i.protected)
    .reduce((s, i) => s + i.size_bytes, 0)
)

const formatScannedAt = (date: Date | null) => {
  if (!date) return "—"
  const pad = (n: number) => String(n).padStart(2, "0")
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

const buildScanMetaLabel = (state: LargeTabScanState, kind: LargeScanKind) => {
  const kindLabel = kind === "folders" ? "大文件夹" : "大文件"
  return `${kindLabel}扫描 ${formatScannedAt(state.scannedAt)} · 共扫描 ${state.filesScanned.toLocaleString("zh-CN")} 个文件`
}

const currentScanMetaLabel = computed(() => buildScanMetaLabel(currentState.value, activeTab.value))

const folderThresholdLabel = computed(
  () => `大文件夹阈值：≥ ${formatThresholdMB(settings.value.largeFolderMB)}`
)
const fileThresholdLabel = computed(
  () => `大文件阈值：≥ ${formatThresholdMB(settings.value.largeFileMB)}`
)
const currentThresholdLabel = computed(() =>
  activeTab.value === "folders" ? folderThresholdLabel.value : fileThresholdLabel.value
)

const scanProgressFilesScannedLabel = computed(() =>
  scanProgressFilesScanned.value.toLocaleString("zh-CN")
)

const scanStatusText = computed(() => {
  if (scanningKind.value === "folders") {
    return `正在全盘查找大文件夹（${folderThresholdLabel.value}）…`
  }
  return `正在全盘查找大文件（${fileThresholdLabel.value}）…，预计需要 10 分钟以上`
})

const emptyTitle = computed(() =>
  activeTab.value === "folders"
    ? "未发现符合阈值的大文件夹"
    : "未发现符合阈值的大文件"
)

const emptySubtitle = computed(() =>
  activeTab.value === "folders"
    ? "可调整文件夹阈值后重新扫描，或切换到「大文件」单独查找"
    : "可调整文件阈值后重新扫描；全盘文件扫描耗时较长，请耐心等待"
)

const goSettings = () => {
  router.push({ name: "Settings", query: { focus: "largeScan" } })
}

const applyScanResult = (
  kind: LargeScanKind,
  result: Awaited<ReturnType<typeof scanLargeFiles>>,
  options: { stoppedEarly?: boolean } = {}
) => {
  const state = tabState(kind).value
  state.items = result.items.map((item) => ({ ...item, selected: false }))
  state.totalBytes = result.summary.total_bytes
  state.filesScanned = result.files_scanned
  state.scannedAt = new Date()
  state.scannedOnce = true

  const kindLabel = kind === "folders" ? "大文件夹" : "大文件"

  if (options.stoppedEarly) {
    if (result.items.length === 0) {
      ElMessage.info(`扫描已结束，暂未发现符合阈值的${kindLabel}`)
    } else {
      ElMessage.success(`扫描已结束，共找到 ${result.items.length} 项${kindLabel}`)
    }
    return
  }

  if (result.items.length === 0) {
    ElMessage.info(`未发现符合阈值的${kindLabel}`)
  }
}

const stopScan = async () => {
  if (!scanning.value || stopping.value) return
  stopping.value = true
  scanStoppedByUser.value = true
  try {
    await cancelScan()
  } catch (e) {
    stopping.value = false
    scanStoppedByUser.value = false
    ElMessage.error(String(e))
  }
}

const runScan = async (kind: LargeScanKind = activeTab.value) => {
  if (!isTauriRuntime() || scanning.value) return

  scanning.value = true
  scanningKind.value = kind
  stopping.value = false
  scanStoppedByUser.value = false
  await resetScanBackend()
  scanProgressCurrentPath.value = ""
  scanProgressFilesScanned.value = 0
  scanProgressItemsFound.value = 0

  void progressUnlisten?.()
  progressUnlisten = await listen<{
    current_path: string
    files_scanned: number
    items_found: number
  }>("large_file_scan_progress", (event) => {
    scanProgressCurrentPath.value = event.payload.current_path
    scanProgressFilesScanned.value = event.payload.files_scanned
    tabState(scanningKind.value).value.filesScanned = event.payload.files_scanned
    if (event.payload.items_found !== scanProgressItemsFound.value) {
      scanProgressItemsFound.value = event.payload.items_found
    }
  })

  try {
    const result = await scanLargeFiles(largeFileBytes.value, largeFolderBytes.value, kind)
    applyScanResult(kind, result, { stoppedEarly: scanStoppedByUser.value })
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    void progressUnlisten?.()
    progressUnlisten = null
    scanning.value = false
    stopping.value = false
    resetScanControlState()
    scanProgressCurrentPath.value = ""
    scanProgressFilesScanned.value = 0
    scanProgressItemsFound.value = 0
  }
}

const startInitialScan = () => {
  activeTab.value = "folders"
  void runScan("folders")
}

const confirmFilesScan = () =>
  ElMessageBox.confirm(
    "大文件扫描需要再次遍历全盘文件，通常需要 10 分钟以上，甚至可能更久。是否现在开始？",
    "大文件扫描耗时较长",
    {
      confirmButtonText: "开始扫描",
      cancelButtonText: "稍后",
      type: "warning",
    }
  )

const onTabChange = async (tab: LargeScanKind) => {
  if (tab === activeTab.value || scanning.value) return

  if (tab === "files" && !tabState("files").value.scannedOnce) {
    try {
      await confirmFilesScan()
    } catch {
      return
    }
    activeTab.value = tab
    await runScan("files")
    return
  }

  activeTab.value = tab
}

const buildDeleteMessage = (targets: LargeScanItem[]) => {
  const list = targets
    .slice(0, 8)
    .map((item) => `· ${item.path}`)
    .join("\n")
  const more = targets.length > 8 ? `\n… 等共 ${targets.length} 项` : ""
  return `以下项将移到废纸篓：\n${list}${more}\n\n请确认这些文件/文件夹可以删除。`
}

const afterCleanSuccess = (result: CleanResult) => {
  const removed = new Set(result.success)
  const state = currentState.value
  state.items = state.items.filter((item) => !removed.has(item.path))
  state.totalBytes = state.items.reduce((sum, item) => sum + item.size_bytes, 0)
}

const deleteTargets = async (targets: LargeScanItem[]) => {
  const deletable = targets.filter((item) => !item.protected)
  if (!deletable.length) return
  await cleanWithConfirm({
    targets: deletable,
    mode: "trash",
    title: "移到废纸篓",
    message: buildDeleteMessage(deletable),
    confirmButtonText: "移到废纸篓",
    onSuccess: afterCleanSuccess,
  })
}

const deleteSingle = async (item: LargeScanItem) => {
  await deleteTargets([item])
}

const handleCleanTrash = async () => {
  const targets = currentItems.value.filter((item) => item.selected && !item.protected)
  if (!targets.length) return
  await deleteTargets(targets)
}

const openInFinder = async (path: string) => {
  try {
    await revealInFinder(path)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const openItemDetail = (item: LargeScanItem) => {
  const focusPath = item.is_directory
    ? item.path
    : item.path.slice(0, item.path.lastIndexOf("/"))

  if (!focusPath) return

  navigateToFolderScan(router, focusPath, {
    highlight: item.is_directory ? undefined : item.path,
  })
}

const copyItemPath = async (path: string) => {
  try {
    await navigator.clipboard.writeText(path)
    ElMessage.success("路径已复制")
  } catch {
    ElMessage.error("复制失败")
  }
}

onUnmounted(() => {
  void progressUnlisten?.()
})
</script>

<style scoped lang="scss">
.large-file-scan {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 100%;
}

.warn-banner {
  display: flex;
  gap: 12px;
  padding: 14px 16px;
  border: 1px solid rgba(245, 158, 11, 0.25);
  background: rgba(245, 158, 11, 0.08);
}

.warn-icon {
  font-size: 18px;
  line-height: 1.4;
}

.warn-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: $color-amber;
}

.warn-desc {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.55;
  color: $text-secondary;
}

.empty-panel .scan-meta {
  margin-top: 12px;
}

.scan-meta {
  display: block;
  margin-top: 6px;
  font-size: 12px;
  color: $text-muted;
  font-variant-numeric: tabular-nums;
}

.scan-panel,
.idle-panel,
.empty-panel {
  flex: 1;
  min-height: 280px;
}

.scan-panel,
.idle-panel,
.empty-panel,
.panel-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: $text-secondary;
}

.scan-panel-tabs,
.idle-panel-tabs,
.empty-panel-tabs {
  margin-bottom: 16px;
}

.scan-progress-lines {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  width: 100%;
  max-width: 640px;
  margin-top: 8px;
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
    padding: 0 16px;
  }

  &--stats {
    font-variant-numeric: tabular-nums;
  }
}

.scan-stat-num {
  font-variant-numeric: tabular-nums;

  &--found {
    display: inline-block;
    min-width: 1.5ch;
    text-align: center;
  }
}

.scan-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 8px;
}

.idle-icon {
  font-size: 40px;
  margin-bottom: 12px;
}

.idle-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: $text-primary;
}

.idle-sub,
.empty-sub {
  margin: 8px 0 0;
  font-size: 13px;
  color: $text-muted;
  max-width: 520px;
}

.idle-scope {
  margin: 12px 0 0;
  font-size: 12px;
  color: $text-muted;
}

.idle-threshold-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  width: 100%;
  max-width: 480px;
  margin-top: 10px;
  padding: 10px 14px;
  border-radius: $radius-button;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid $border-subtle;
}

.idle-threshold-label {
  font-size: 12px;
  color: $text-secondary;
  font-variant-numeric: tabular-nums;
}

.idle-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 18px;
}

.empty-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: $text-primary;
}

.empty-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 16px;
}

.result-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid $border-subtle;
}

.result-summary {
  min-width: 0;
  flex: 1;
}

.result-summary-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin: 0;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.summary-size {
  font-size: 16px;
  font-weight: 700;
  color: $text-primary;
  flex-shrink: 0;
}

.summary-divider {
  color: $text-muted;
  flex-shrink: 0;
}

.summary-detail {
  font-size: 12px;
  color: $text-muted;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.result-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.scan-item {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);

  &:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  &.is-protected {
    opacity: 0.72;
  }
}

.item-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.item-main {
  min-width: 0;
}

.item-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.item-name {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-tag {
  display: inline-flex;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 500;
  background: rgba(59, 130, 246, 0.15);
  color: $color-blue;
  flex-shrink: 0;

  &.is-system {
    background: rgba(239, 68, 68, 0.15);
    color: $color-red;
  }

  &.is-uninstalled {
    background: rgba(245, 158, 11, 0.15);
    color: $color-amber;
  }
}

.item-path {
  display: block;
  width: 100%;
  margin: 4px 0 0;
  padding: 0;
  border: none;
  background: transparent;
  text-align: left;
  font-size: 11px;
  line-height: 1.45;
  color: $text-muted;
  word-break: break-all;
  cursor: pointer;

  &:hover {
    color: $text-secondary;
  }
}

.item-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.item-time {
  font-size: 10px;
  color: $text-muted;
  font-variant-numeric: tabular-nums;
}

.item-size {
  font-size: 12px;
  font-weight: 600;
  color: $text-secondary;
  font-variant-numeric: tabular-nums;
}

.item-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.path-action {
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid $border-subtle;
  background: rgba(255, 255, 255, 0.04);
  color: $text-secondary;
  font-size: 10px;
  cursor: pointer;

  &:hover:not(:disabled) {
    color: $text-primary;
    border-color: rgba(124, 58, 237, 0.35);
  }

  &--danger:hover:not(:disabled) {
    color: $color-red;
    border-color: rgba(239, 68, 68, 0.35);
  }

  &:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
}

.cleaning-overlay {
  position: absolute;
  inset: 0;
  background: rgba(13, 13, 18, 0.72);
  backdrop-filter: blur(4px);
  z-index: 2;
  gap: 10px;
  color: $text-secondary;
}
</style>
