<template>
  <div class="junk-clean">
    <!-- 扫描中 -->
    <div v-if="scanning" class="scanning-view">
      <DiskRing
        :percent="scanProgress?.percent ?? 0"
        :label="lastScanMode === 'deep' ? '深度扫描' : '智能扫描'"
        :scanning="true"
      />
      <p class="scanning-text">{{ lastScanMode === 'deep' ? '深度扫描中...' : '智能扫描中...' }}</p>
    </div>

    <!-- 扫描结果 -->
    <template v-else-if="scanItems.length > 0">
      <div class="result-panel" :class="{ 'is-busy': cleaning }">
        <div class="result-header">
          <div class="result-summary">
            <span class="summary-size">{{ formatSize(totalSelectedBytes) }}</span>
            <span class="summary-label">
              已选 / 共 {{ formatSize(totalBytes) }} 可释放 · {{ scanItems.length }} 项
              <template v-if="lastScanLabel"> · {{ lastScanLabel }}</template>
            </span>
          </div>
          <div class="result-actions">
            <button
              class="action-btn action-btn--ghost"
              :disabled="scanning || cleaning"
              @click="rescan"
            >
              重新扫描
            </button>
            <button
              class="action-btn action-btn--ghost"
              :disabled="cleaning || selectedCount === 0"
              @click="deselectAll"
            >
              取消全选
            </button>
            <button
              class="action-btn action-btn--primary"
              :disabled="selectedCount === 0 || cleaning"
              @click="handleCleanTrash"
            >
              移到废纸篓 ({{ selectedCount }})
            </button>
            <button
              class="action-btn action-btn--danger"
              :disabled="selectedCount === 0 || cleaning"
              @click="handleCleanPermanent"
            >
              永久删除
            </button>
          </div>
        </div>

        <div class="result-list">
          <div
            v-for="group in groupedItems"
            :key="group.category"
            class="group-card glass-card"
          >
            <div class="group-header" @click="toggleGroup(group.category)">
              <el-checkbox
                class="group-select"
                :model-value="isGroupAllSelected(group)"
                :indeterminate="isGroupIndeterminate(group)"
                :disabled="cleaning"
                @click.stop
                @change="(checked: boolean) => setGroupSelection(group, checked)"
              />
              <span class="group-dot" :class="group.category" />
              <span class="group-title">{{ group.label }}</span>
              <span class="group-size">{{ formatSize(group.totalSize) }}</span>
              <el-icon class="group-arrow" :class="{ expanded: expandedGroups.has(group.category) }">
                <ArrowDown />
              </el-icon>
            </div>

            <Transition name="expand">
              <div v-show="expandedGroups.has(group.category)" class="group-items">
                <div v-for="item in group.items" :key="item.id" class="scan-item">
                  <el-checkbox v-model="item.selected" :disabled="cleaning" />
                  <div class="item-info">
                    <div class="item-title-row">
                      <span class="item-title">{{ item.title }}</span>
                      <span
                        v-if="item.app_label"
                        class="app-tag"
                        :class="{ 'is-uninstalled': item.app_installed === false }"
                        :title="item.bundle_id ? `Bundle ID: ${item.bundle_id}` : item.app_label"
                      >
                        {{ item.app_label }}
                      </span>
                    </div>
                    <div class="item-path">{{ item.path }}</div>
                  </div>
                  <span class="item-size">{{ formatSize(item.size_bytes) }}</span>
                  <span class="risk-tag" :class="item.risk">{{ riskLabel(item.risk) }}</span>
                </div>
              </div>
            </Transition>
          </div>
        </div>

        <Transition name="page-fade">
          <div v-if="cleaning" class="cleaning-overlay flex-c-c">
            <DiskRing :percent="100" :label="cleaningText" :scanning="true" />
            <p class="cleaning-detail">{{ cleaningDetail }}</p>
            <p class="cleaning-hint">大体积缓存可能需要较长时间，请稍候</p>
          </div>
        </Transition>
      </div>
    </template>

    <!-- 扫描完成，无垃圾 -->
    <div v-else-if="isCleanResult" class="clean-result-state">
      <DiskRing :percent="100" label="状态良好" sub-label="暂无垃圾" />
      <h2 class="clean-title">很干净！</h2>
      <p class="clean-text">未发现可清理的缓存、日志或垃圾文件</p>
      <p v-if="lastScanLabel" class="clean-meta">{{ lastScanLabel }}</p>
      <div class="empty-actions">
        <el-tooltip placement="top" popper-class="scan-scope-tooltip">
          <template #content>
            <div class="scan-scope-tip">{{ QUICK_SCAN_SCOPE_TIP }}</div>
          </template>
          <button class="action-btn action-btn--primary" :disabled="scanning" @click="scan('quick')">
            快速扫描
          </button>
        </el-tooltip>
        <el-tooltip placement="top" popper-class="scan-scope-tooltip">
          <template #content>
            <div class="scan-scope-tip">{{ DEEP_SCAN_SCOPE_TIP }}</div>
          </template>
          <button class="action-btn action-btn--ghost" :disabled="scanning" @click="scan('deep')">
            深度扫描
          </button>
        </el-tooltip>
      </div>
    </div>

    <!-- 未扫描 -->
    <div v-else class="empty-state">
      <DiskRing :percent="0" label="等待扫描" />
      <p class="empty-text">点击「智能扫描」开始查找可清理项</p>
      <div class="empty-actions">
        <el-tooltip placement="top" popper-class="scan-scope-tooltip">
          <template #content>
            <div class="scan-scope-tip">{{ QUICK_SCAN_SCOPE_TIP }}</div>
          </template>
          <button class="gradient-btn" @click="scan('quick')">快速扫描</button>
        </el-tooltip>
        <el-tooltip placement="top" popper-class="scan-scope-tooltip">
          <template #content>
            <div class="scan-scope-tip">{{ DEEP_SCAN_SCOPE_TIP }}</div>
          </template>
          <button class="secondary-btn" @click="scan('deep')">深度扫描</button>
        </el-tooltip>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onUnmounted, reactive, ref } from "vue"
import { ArrowDown } from "@element-plus/icons-vue"
import { ElMessage, ElMessageBox } from "element-plus"
import DiskRing from "@/components/DiskRing.vue"
import { useScanner } from "@/composables/useScanner"
import { cleanPaths } from "@/api/cleaner"
import { formatSize } from "@/utils/formatSize"
import { notifyCleanResult } from "@/utils/cleanResult"
import { QUICK_SCAN_SCOPE_TIP, DEEP_SCAN_SCOPE_TIP } from "@/constants/scanScopeTips"
import type { ScanItem } from "@/types/cleaner"

const { scanning, scanProgress, scanItems, lastScanMode, lastScanAt, scan, rescan, teardownListeners } = useScanner()
const expandedGroups = reactive(new Set<string>(["cache", "log", "trash", "leftover", "large_file"]))
const cleaning = ref(false)
const cleaningText = ref("")
const cleaningDetail = ref("")

const categoryLabels: Record<string, string> = {
  cache: "应用缓存",
  log: "系统日志",
  trash: "废纸篓",
  leftover: "第三方遗留",
  large_file: "大文件",
}

onUnmounted(() => teardownListeners())

const lastScanLabel = computed(() => {
  if (!lastScanAt.value) return ""
  const mode = lastScanMode.value === "deep" ? "深度扫描" : "快速扫描"
  const time = new Date(lastScanAt.value).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  })
  return `${mode}于 ${time}`
})

const isCleanResult = computed(
  () => !scanning.value && !cleaning.value && scanItems.value.length === 0 && lastScanAt.value !== null
)

const totalBytes = computed(() => scanItems.value.reduce((s, i) => s + i.size_bytes, 0))
const selectedCount = computed(() => scanItems.value.filter((i) => i.selected).length)
const totalSelectedBytes = computed(() =>
  scanItems.value.filter((i) => i.selected).reduce((s, i) => s + i.size_bytes, 0)
)

type ScanItemGroup = {
  category: string
  label: string
  items: ScanItem[]
  totalSize: number
}

const groupedItems = computed(() => {
  const map = new Map<string, ScanItem[]>()
  for (const item of scanItems.value) {
    const list = map.get(item.category) ?? []
    list.push(item)
    map.set(item.category, list)
  }
  return Array.from(map.entries()).map(([category, items]) => ({
    category,
    label: categoryLabels[category] ?? category,
    items,
    totalSize: items.reduce((s, i) => s + i.size_bytes, 0),
  }))
})

const toggleGroup = (cat: string) => {
  if (expandedGroups.has(cat)) expandedGroups.delete(cat)
  else expandedGroups.add(cat)
}

const isGroupAllSelected = (group: ScanItemGroup) =>
  group.items.length > 0 && group.items.every((i) => i.selected)

const isGroupIndeterminate = (group: ScanItemGroup) => {
  const n = group.items.filter((i) => i.selected).length
  return n > 0 && n < group.items.length
}

const setGroupSelection = (group: ScanItemGroup, selected: boolean) => {
  group.items.forEach((i) => {
    i.selected = selected
  })
}

const deselectAll = () => {
  scanItems.value.forEach((i) => {
    i.selected = false
  })
}

const riskLabel = (risk: string) => ({ low: "低风险", medium: "中风险", high: "高风险" }[risk] ?? risk)

const getSelectedPaths = () => scanItems.value.filter((i) => i.selected).map((i) => i.path)

const removeCleanedItems = (successPaths: string[]) => {
  const removed = new Set(successPaths)
  scanItems.value = scanItems.value.filter((i) => !removed.has(i.path))
}

const refreshAfterClean = async (successPaths: string[]) => {
  removeCleanedItems(successPaths)
  if (scanItems.value.length === 0) {
    await rescan()
  }
}

const runClean = async (mode: "trash" | "permanent", paths: string[], sizeLabel: string) => {
  const selectedItems = scanItems.value.filter((i) => i.selected && paths.includes(i.path))
  const sizes = selectedItems.map((i) => i.size_bytes)

  cleaning.value = true
  cleaningText.value = mode === "trash" ? "正在移到废纸篓" : "正在永久删除"
  cleaningDetail.value = `处理 ${paths.length} 项 · ${sizeLabel}`

  try {
    const result = await cleanPaths(paths, mode, sizes)
    await notifyCleanResult(result, mode === "trash" ? "移到废纸篓" : "永久删除")
    await refreshAfterClean(result.success)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    cleaning.value = false
    cleaningText.value = ""
    cleaningDetail.value = ""
  }
}

const handleCleanTrash = async () => {
  const paths = getSelectedPaths()
  if (paths.length === 0) return

  try {
    await ElMessageBox.confirm(
      `即将把 ${paths.length} 项（${formatSize(totalSelectedBytes.value)}）移到废纸篓，是否继续？`,
      "确认清理",
      { confirmButtonText: "移到废纸篓", cancelButtonText: "取消", type: "warning" }
    )
    await runClean("trash", paths, formatSize(totalSelectedBytes.value))
  } catch {
    // 用户取消
  }
}

const handleCleanPermanent = async () => {
  const paths = getSelectedPaths()
  if (paths.length === 0) return

  try {
    await ElMessageBox.confirm(
      `即将永久删除 ${paths.length} 项（${formatSize(totalSelectedBytes.value)}），此操作不可从废纸篓恢复。`,
      "永久删除确认",
      { confirmButtonText: "继续", cancelButtonText: "取消", type: "warning" }
    )
    await ElMessageBox.confirm(
      `请再次确认：永久删除 ${paths.length} 项后无法恢复，确定要继续吗？`,
      "最终确认",
      {
        confirmButtonText: "永久删除",
        cancelButtonText: "取消",
        type: "error",
        confirmButtonClass: "el-button--danger",
      }
    )
    await runClean("permanent", paths, formatSize(totalSelectedBytes.value))
  } catch {
    // 用户取消
  }
}
</script>

<style scoped lang="scss">
.junk-clean {
  padding-top: 8px;
}

.result-panel {
  position: relative;
  min-height: 320px;

  &.is-busy {
    .result-list,
    .result-header {
      pointer-events: none;
      user-select: none;
    }
  }
}

.cleaning-overlay {
  position: absolute;
  inset: 0;
  z-index: 10;
  flex-direction: column;
  gap: 12px;
  border-radius: $radius-card;
  background: rgba(13, 13, 18, 0.82);
  backdrop-filter: blur(8px);
}

.cleaning-detail {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
  color: $text-primary;
}

.cleaning-hint {
  margin: 0;
  font-size: 12px;
  color: $text-muted;
}

.flex-c-c {
  display: flex;
  align-items: center;
  justify-content: center;
}

.scanning-view,
.empty-state,
.clean-result-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 40px 0;
}

.clean-result-state {
  padding: 48px 0 40px;
}

.clean-title {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: $color-green;
}

.clean-text {
  margin: 0;
  font-size: 14px;
  color: $text-secondary;
}

.clean-meta {
  margin: 0;
  font-size: 12px;
  color: $text-muted;
}

.scanning-text,
.empty-text {
  color: $text-secondary;
  font-size: 14px;
}

.empty-actions {
  display: flex;
  gap: 12px;
}

.secondary-btn {
  padding: 14px 24px;
  border-radius: $radius-button;
  border: 1px solid $border-subtle;
  background: $bg-card;
  color: $text-secondary;
  cursor: pointer;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.summary-size {
  font-size: 28px;
  font-weight: 700;
  color: $color-green;
  margin-right: 10px;
}

.summary-label {
  font-size: 13px;
  color: $text-muted;
}

.result-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.result-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.group-card {
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 18px;
  cursor: pointer;
  user-select: none;

  &:hover { background: $bg-card-hover; }
}

.group-select {
  flex-shrink: 0;
}

.group-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: $color-purple;

  &.log { background: $color-amber; }
  &.trash { background: $color-red; }
  &.leftover { background: $color-blue; }
  &.large_file { background: $color-green; }
}

.group-title {
  flex: 1;
  font-weight: 600;
  color: $text-primary;
}

.group-size {
  font-size: 14px;
  font-weight: 600;
  color: $text-secondary;
}

.group-arrow {
  transition: transform 0.2s ease;
  color: $text-muted;

  &.expanded { transform: rotate(180deg); }
}

.group-items {
  border-top: 1px solid $border-subtle;
}

.scan-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);

  &:last-child { border-bottom: none; }
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.item-title {
  font-size: 13px;
  font-weight: 500;
  color: $text-primary;
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
  font-size: 11px;
  color: $text-muted;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 2px;
}

.item-size {
  font-size: 13px;
  font-weight: 600;
  color: $text-secondary;
  flex-shrink: 0;
}

.expand-enter-active,
.expand-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
}
</style>
