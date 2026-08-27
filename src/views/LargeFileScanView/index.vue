<template>
  <div class="large-file-scan">
    <div v-if="scannedOnce && !scanning && items.length" class="warn-banner glass-card">
      <span class="warn-icon">⚠️</span>
      <div class="warn-text">
        <p class="warn-title">删除前请谨慎确认</p>
        <p class="warn-desc">
          扫描结果可能包含仍在使用的文件或系统目录，请自行判断后再操作。本页仅支持移到废纸篓，可从废纸篓恢复。
        </p>
      </div>
    </div>

    <div v-if="scanning" class="scan-panel glass-card panel-loading">
      <el-icon class="is-loading" :size="36"><Loading /></el-icon>
      <div class="scan-progress-lines">
        <p class="scan-status">
          {{ scanPaused ? "扫描已暂停" : `正在全盘查找大文件（${largeScanThresholdLabel}）…` }}
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
      v-else-if="items.length"
      class="result-panel glass-card"
      :class="{ 'is-busy': cleaning }"
    >
      <div class="result-header">
        <div class="result-summary" :title="scanMetaLabel">
          <p class="result-summary-line">
            <span class="summary-size">{{ formatSize(selectedBytes) }}</span>
            <span class="summary-divider">|</span>
            <span class="summary-detail">已选 / 共 {{ formatSize(totalBytes) }} · {{ items.length }} 项</span>
          </p>
        </div>
        <div class="result-actions">
          <button class="action-btn action-btn--ghost" :disabled="cleaning" @click="runScan">
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
                :class="{ 'is-uninstalled': item.app_installed === false }"
                :title="item.bundle_id ? `Bundle ID: ${item.bundle_id}` : item.app_label"
              >
                {{ item.app_label }}
              </span>
            </div>
            <p class="item-path" :title="item.path">{{ item.path }}</p>
          </div>
          <div class="item-meta">
            <span class="item-time">{{ item.modified }}</span>
            <span class="item-size">{{ formatSize(item.size_bytes) }}</span>
          </div>
          <div class="item-actions">
            <button class="path-action" @click="openInFinder(item.path)">访达</button>
            <button
              v-if="!item.protected"
              class="path-action path-action--danger"
              :disabled="cleaning"
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

    <div v-else-if="scannedOnce && !scanning" class="empty-panel glass-card">
      <p class="empty-title">未发现符合阈值的大文件或文件夹</p>
      <p class="empty-sub">可调整阈值后重新扫描，或检查扫描范围是否覆盖目标路径</p>
      <p class="scan-meta">{{ scanMetaLabel }}</p>
      <button class="action-btn action-btn--primary" @click="goSettings">调整阈值</button>
    </div>

    <div v-else class="idle-panel glass-card">
      <span class="idle-icon">🔍</span>
      <p class="idle-title">查找占用空间的大文件与文件夹</p>
      <p class="idle-sub">点击「开始扫描」后将在后台遍历主目录、应用程序、系统与磁盘卷</p>
      <p class="idle-scope">扫描范围：主目录 / 应用程序 / 系统 / 磁盘卷</p>
      <div class="idle-threshold-row">
        <span class="idle-threshold-label">{{ largeScanThresholdLabel }}</span>
        <button class="pill-btn pill-btn--compact" @click="goSettings">阈值配置</button>
      </div>
      <div class="idle-actions">
        <button class="action-btn action-btn--primary" @click="runScan">开始扫描</button>
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
import { ElMessage } from "element-plus"
import ScanControlButtons from "@/components/ScanControlButtons.vue"
import { scanLargeFiles } from "@/api/folder"
import { useAppSettings } from "@/composables/useAppSettings"
import { useCleanAction } from "@/composables/useCleanAction"
import { useScanControl } from "@/composables/useScanControl"
import { revealInFinder } from "@/utils/finder"
import { formatSize } from "@/utils/formatSize"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import type { BrowseItem, CleanResult } from "@/types/cleaner"

type ScanItem = BrowseItem & { selected?: boolean }

const router = useRouter()
const { largeFileBytes, largeFolderBytes, largeScanThresholdLabel } = useAppSettings()
const { scanPaused, resetScanControlState, resetScanBackend, pauseScan, resumeScan, cancelScan } =
  useScanControl()
const { cleaning, cleaningText, cleanWithConfirm } = useCleanAction()

const scanning = ref(false)
const stopping = ref(false)
const scanStoppedByUser = ref(false)
const scannedOnce = ref(false)
const items = ref<ScanItem[]>([])
const totalBytes = ref(0)
const filesScanned = ref(0)
const scannedAt = ref<Date | null>(null)
const scanProgressCurrentPath = ref("")
const scanProgressFilesScanned = ref(0)
const scanProgressItemsFound = ref(0)
let progressUnlisten: UnlistenFn | null = null

const sortedItems = computed(() =>
  [...items.value].sort((a, b) => b.size_bytes - a.size_bytes || a.path.localeCompare(b.path))
)
const selectedCount = computed(() => items.value.filter((i) => i.selected && !i.protected).length)
const selectedBytes = computed(() =>
  items.value.filter((i) => i.selected && !i.protected).reduce((s, i) => s + i.size_bytes, 0)
)
const scannedAtLabel = computed(() => {
  if (!scannedAt.value) return "—"
  const d = scannedAt.value
  const pad = (n: number) => String(n).padStart(2, "0")
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
})
const filesScannedLabel = computed(() => filesScanned.value.toLocaleString("zh-CN"))
const scanProgressFilesScannedLabel = computed(() =>
  scanProgressFilesScanned.value.toLocaleString("zh-CN")
)
const scanMetaLabel = computed(() =>
  `本次扫描 ${scannedAtLabel.value} · 共扫描 ${filesScannedLabel.value} 个文件`
)

const goSettings = () => {
  router.push({ name: "Settings", query: { focus: "largeScan" } })
}

const applyScanResult = (
  result: Awaited<ReturnType<typeof scanLargeFiles>>,
  options: { stoppedEarly?: boolean } = {}
) => {
  items.value = result.items.map((item) => ({ ...item, selected: false }))
  totalBytes.value = result.summary.total_bytes
  filesScanned.value = result.files_scanned
  scannedAt.value = new Date()
  scannedOnce.value = true

  if (options.stoppedEarly) {
    if (result.items.length === 0) {
      ElMessage.info("扫描已结束，暂未发现符合阈值的大文件或文件夹")
    } else {
      ElMessage.success(`扫描已结束，共找到 ${result.items.length} 项`)
    }
    return
  }

  if (result.items.length === 0) {
    ElMessage.info(`未发现符合 ${largeScanThresholdLabel.value} 的文件或文件夹`)
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

const runScan = async () => {
  if (!isTauriRuntime() || scanning.value) return

  scanning.value = true
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
    filesScanned.value = event.payload.files_scanned
    if (event.payload.items_found !== scanProgressItemsFound.value) {
      scanProgressItemsFound.value = event.payload.items_found
    }
  })

  try {
    const result = await scanLargeFiles(largeFileBytes.value, largeFolderBytes.value)
    applyScanResult(result, { stoppedEarly: scanStoppedByUser.value })
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

const buildDeleteMessage = (targets: ScanItem[]) => {
  const list = targets
    .slice(0, 8)
    .map((item) => `· ${item.path}`)
    .join("\n")
  const more = targets.length > 8 ? `\n… 等共 ${targets.length} 项` : ""
  return `以下项将移到废纸篓：\n${list}${more}\n\n请确认这些文件/文件夹可以删除。`
}

const afterCleanSuccess = (result: CleanResult) => {
  const removed = new Set(result.success)
  items.value = items.value.filter((item) => !removed.has(item.path))
  totalBytes.value = items.value.reduce((sum, item) => sum + item.size_bytes, 0)
}

const deleteTargets = async (targets: ScanItem[]) => {
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

const deleteSingle = async (item: ScanItem) => {
  await deleteTargets([item])
}

const handleCleanTrash = async () => {
  const targets = items.value.filter((item) => item.selected && !item.protected)
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
  max-width: 480px;
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

.empty-panel .action-btn {
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

  &.is-uninstalled {
    background: rgba(245, 158, 11, 0.15);
    color: $color-amber;
  }
}

.item-path {
  margin: 4px 0 0;
  font-size: 11px;
  line-height: 1.45;
  color: $text-muted;
  word-break: break-all;
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
