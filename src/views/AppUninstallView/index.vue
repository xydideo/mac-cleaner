<template>
  <div class="app-uninstall">
    <div class="uninstall-layout">
      <!-- 左侧：应用列表 -->
      <div class="app-list-panel glass-card">
        <div class="panel-header">
          <div class="panel-header-left">
            <h3>已安装应用</h3>
            <span class="app-count">{{ filteredUserApps.length }} 个可卸载</span>
            <span v-if="appsCacheLabel" class="cache-label">{{ appsCacheLabel }}</span>
          </div>
          <button
            class="action-btn action-btn--ghost"
            :disabled="loadingApps"
            @click="refreshApps"
          >
            刷新
          </button>
        </div>
        <div class="search-box">
          <input
            v-model="keyword"
            class="search-input"
            type="search"
            placeholder="搜索应用名称..."
          />
        </div>

        <div v-if="loadingApps" class="panel-loading">
          <el-icon class="is-loading" :size="28"><Loading /></el-icon>
          <span>正在加载应用列表...</span>
        </div>

        <div v-else class="app-list">
          <!-- 用户安装的应用：按体积从大到小 -->
          <button
            v-for="app in filteredUserApps"
            :key="app.path"
            class="app-item"
            :class="{ active: selectedApp?.path === app.path }"
            @click="selectApp(app)"
          >
            <img v-if="app.icon_base64" :src="app.icon_base64" :alt="app.name" class="app-icon" />
            <div v-else class="app-avatar">{{ app.name.charAt(0).toUpperCase() }}</div>
            <div class="app-meta">
              <div class="app-name">{{ app.name }}</div>
              <div class="app-sub">{{ app.version }} · {{ formatSize(app.size_bytes) }}</div>
            </div>
          </button>

          <!-- 分隔线：系统应用 -->
          <div v-if="filteredUserApps.length && filteredSystemApps.length" class="list-divider">
            <span class="divider-line" />
            <span class="divider-text">系统应用（不可卸载）</span>
            <span class="divider-line" />
          </div>

          <button
            v-for="app in filteredSystemApps"
            :key="app.path"
            class="app-item is-system"
            disabled
          >
            <img v-if="app.icon_base64" :src="app.icon_base64" :alt="app.name" class="app-icon is-dim" />
            <div v-else class="app-avatar is-dim">{{ app.name.charAt(0).toUpperCase() }}</div>
            <div class="app-meta">
              <div class="app-name">{{ app.name }}</div>
              <div class="app-sub">{{ app.version }} · {{ formatSize(app.size_bytes) }}</div>
            </div>
            <span class="system-tag">系统</span>
          </button>
        </div>
      </div>

      <!-- 右侧：关联文件 -->
      <div class="related-panel glass-card">
        <template v-if="!selectedApp">
          <div class="related-empty">
            <div class="empty-icon">📦</div>
            <h3>选择要卸载的应用</h3>
            <p>从左侧列表选择一个应用，扫描其关联文件后完整卸载</p>
          </div>
        </template>

        <template v-else>
          <div class="panel-header">
            <div>
              <h3>{{ selectedApp.name }}</h3>
              <p class="bundle-id">{{ selectedApp.bundle_id }}</p>
            </div>
            <button
              class="action-btn action-btn--ghost"
              :disabled="scanningRelated"
              @click="scanRelated(true)"
            >
              重新扫描
            </button>
          </div>

          <div v-if="scanningRelated" class="panel-loading">
            <el-icon class="is-loading" :size="28"><Loading /></el-icon>
            <span>{{ scanPaused ? "扫描已暂停" : "正在扫描关联文件..." }}</span>
            <ScanControlButtons :paused="scanPaused" @pause="pauseScan" @resume="resumeScan" />
          </div>

          <template v-else-if="relatedItems.length > 0">
            <div class="related-summary">
              预计释放 <strong>{{ formatSize(totalSelectedBytes) }}</strong>
              / {{ formatSize(totalRelatedBytes) }} · 已选 {{ selectedCount }} 项
            </div>

            <div class="related-list">
              <div v-for="item in relatedItems" :key="item.id" class="related-item">
                <el-checkbox v-model="item.selected" :disabled="uninstalling" />
                <div class="item-info">
                  <div class="item-title-row">
                    <span class="item-title">{{ item.title }}</span>
                    <span class="category-tag">{{ categoryLabel(item.category) }}</span>
                  </div>
                  <div class="item-path" :title="item.path">{{ item.path }}</div>
                  <div class="item-desc">{{ item.description }}</div>
                </div>
                <div class="item-side">
                  <div class="item-side-top">
                    <span class="item-size">{{ formatSize(item.size_bytes) }}</span>
                    <span class="risk-tag" :class="item.risk">{{ riskLabel(item.risk) }}</span>
                  </div>
                  <div class="item-side-actions">
                    <button class="path-action" @click.stop="goAnalyzePath(item.path)">详细</button>
                    <button class="path-action" title="在访达中显示" @click.stop="openInFinder(item.path)">
                      访达
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <div class="related-actions">
              <button
                class="action-btn action-btn--primary"
                :disabled="selectedCount === 0 || uninstalling"
                @click="handleUninstall('trash')"
              >
                卸载到废纸篓 ({{ selectedCount }})
              </button>
              <button
                class="action-btn action-btn--danger"
                :disabled="selectedCount === 0 || uninstalling"
                @click="handleUninstall('permanent')"
              >
                永久卸载
              </button>
            </div>
          </template>

          <div v-else class="related-empty">
            <p>未找到关联文件，仅可卸载主程序</p>
          </div>
        </template>

        <Transition name="page-fade">
          <div v-if="uninstalling" class="uninstall-overlay flex-c-c">
            <el-icon class="is-loading" :size="36"><Loading /></el-icon>
            <p>{{ uninstallText }}</p>
          </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: "AppUninstall" })

import { computed, onActivated, onDeactivated, onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import { Loading } from "@element-plus/icons-vue"
import { ElMessage, ElMessageBox } from "element-plus"
import { cleanPaths } from "@/api/cleaner"
import ScanControlButtons from "@/components/ScanControlButtons.vue"
import { useScanControl } from "@/composables/useScanControl"
import { formatCacheAge, useAppUninstallCache } from "@/composables/useAppUninstallCache"
import { revealInFinder } from "@/utils/finder"
import { navigateToFolderScan } from "@/utils/folderNavigation"
import { formatSize } from "@/utils/formatSize"
import { notifyCleanResult } from "@/utils/cleanResult"
import type { AppInfo, AppRelatedItem } from "@/types/cleaner"

const router = useRouter()
const { scanPaused, resetScanBackend, pauseScan, resumeScan } = useScanControl()
const { loadApps, loadRelated, removeAppFromCache, invalidateAll, getAppsCachedAt, isAppsCacheFresh, saveUiState, getUiState } =
  useAppUninstallCache()

const loadingApps = ref(!isAppsCacheFresh())
const apps = ref<AppInfo[]>([])
const keyword = ref("")
const selectedApp = ref<AppInfo | null>(null)
const relatedItems = ref<AppRelatedItem[]>([])
const scanningRelated = ref(false)
const uninstalling = ref(false)
const uninstallText = ref("")
const appsCachedAt = ref<number | null>(null)

const appsCacheLabel = computed(() => formatCacheAge(appsCachedAt.value))

const categoryLabels: Record<string, string> = {
  main: "主程序",
  cache: "缓存",
  support: "数据",
  container: "沙盒",
  preferences: "偏好",
  saved_state: "状态",
  logs: "日志",
  webkit: "Web",
  http: "网络",
  launch_agent: "启动项",
  cookies: "Cookie",
}

const filteredApps = computed(() => {
  const q = keyword.value.trim().toLowerCase()
  if (!q) return apps.value
  return apps.value.filter(
    (a) => a.name.toLowerCase().includes(q) || a.bundle_id.toLowerCase().includes(q)
  )
})

const filteredUserApps = computed(() => filteredApps.value.filter((a) => !a.is_system_app))
const filteredSystemApps = computed(() => filteredApps.value.filter((a) => a.is_system_app))

const selectedCount = computed(() => relatedItems.value.filter((i) => i.selected).length)
const totalRelatedBytes = computed(() => relatedItems.value.reduce((s, i) => s + i.size_bytes, 0))
const totalSelectedBytes = computed(() =>
  relatedItems.value.filter((i) => i.selected).reduce((s, i) => s + i.size_bytes, 0)
)

onMounted(() => {
  void bootstrapPage()
})

onActivated(() => {
  void bootstrapPage()
})

onDeactivated(() => {
  saveUiState({
    keyword: keyword.value,
    selectedAppPath: selectedApp.value?.path ?? null,
  })
})

const restoreUiState = async () => {
  const state = getUiState()
  if (!state) return
  keyword.value = state.keyword
  if (!state.selectedAppPath || selectedApp.value) return
  const app = apps.value.find((a) => a.path === state.selectedAppPath)
  if (!app) return
  selectedApp.value = app
  await scanRelated(false)
}

const bootstrapPage = async () => {
  if (isAppsCacheFresh() && apps.value.length > 0) {
    loadingApps.value = false
    appsCachedAt.value = getAppsCachedAt()
    return
  }
  await loadAppList(false)
  await restoreUiState()
}

const loadAppList = async (force: boolean) => {
  if (!force && isAppsCacheFresh()) {
    loadingApps.value = false
    apps.value = await loadApps(false)
    appsCachedAt.value = getAppsCachedAt()
    return
  }
  loadingApps.value = true
  try {
    apps.value = await loadApps(force)
    appsCachedAt.value = getAppsCachedAt()
  } finally {
    loadingApps.value = false
  }
}

const refreshApps = async () => {
  invalidateAll()
  selectedApp.value = null
  relatedItems.value = []
  await loadAppList(true)
}

const selectApp = async (app: AppInfo) => {
  if (app.is_system_app) return
  selectedApp.value = app
  await scanRelated(false)
}

const scanRelated = async (force = false) => {
  if (!selectedApp.value) return
  scanningRelated.value = true
  if (force) {
    relatedItems.value = []
  }
  try {
    await resetScanBackend()
    const items = await loadRelated(selectedApp.value, force)
    relatedItems.value = items.map((item) => ({
      ...item,
      selected: defaultSelected(item),
    }))
  } finally {
    scanningRelated.value = false
  }
}

const defaultSelected = (item: AppRelatedItem) => {
  if (item.risk === "high") return false
  return true
}

const categoryLabel = (cat: string) => categoryLabels[cat] ?? cat
const riskLabel = (risk: string) =>
  ({ low: "低", medium: "中", high: "高" }[risk] ?? risk)

const openInFinder = async (path: string) => {
  try {
    await revealInFinder(path)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const goAnalyzePath = (path: string) => {
  navigateToFolderScan(router, path)
}

const handleUninstall = async (mode: "trash" | "permanent") => {
  const selected = relatedItems.value.filter((i) => i.selected)
  if (selected.length === 0) return

  const paths = selected.map((i) => i.path)
  const sizes = selected.map((i) => i.size_bytes)
  const sizeLabel = formatSize(totalSelectedBytes.value)
  const appName = selectedApp.value?.name ?? "应用"

  try {
    if (mode === "trash") {
      await ElMessageBox.confirm(
        `即将卸载「${appName}」及其 ${paths.length} 项关联文件（${sizeLabel}），移到废纸篓？`,
        "确认卸载",
        { confirmButtonText: "卸载到废纸篓", cancelButtonText: "取消", type: "warning" }
      )
    } else {
      await ElMessageBox.confirm(
        `即将永久卸载「${appName}」及 ${paths.length} 项关联文件（${sizeLabel}），不可恢复。`,
        "永久卸载确认",
        { confirmButtonText: "继续", cancelButtonText: "取消", type: "warning" }
      )
      await ElMessageBox.confirm(
        `请再次确认：永久卸载「${appName}」后无法恢复，确定继续吗？`,
        "最终确认",
        {
          confirmButtonText: "永久卸载",
          cancelButtonText: "取消",
          type: "error",
          confirmButtonClass: "el-button--danger",
        }
      )
    }

    uninstalling.value = true
    uninstallText.value = mode === "trash" ? "正在卸载到废纸篓..." : "正在永久卸载..."

    const result = await cleanPaths(paths, mode, sizes)
    await notifyCleanResult(result, mode === "trash" ? "卸载到废纸篓" : "永久卸载")

    // 从列表移除已卸载应用
    if (result.success.length > 0 && selectedApp.value) {
      removeAppFromCache(selectedApp.value)
      apps.value = apps.value.filter((a) => a.path !== selectedApp.value!.path)
      appsCachedAt.value = getAppsCachedAt()
    }
    selectedApp.value = null
    relatedItems.value = []
  } catch (e) {
    if (e === "cancel") return
    ElMessage.error(String(e))
  } finally {
    uninstalling.value = false
    uninstallText.value = ""
  }
}
</script>

<style scoped lang="scss">
.app-uninstall {
  padding-top: 8px;
  height: calc(100vh - #{$header-height} - 28px);
}

.uninstall-layout {
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 14px;
  height: 100%;
  min-height: 0;
}

.app-list-panel,
.related-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 18px 12px;
  flex-shrink: 0;
  gap: 12px;

  h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: $text-primary;
  }
}

.panel-header-left {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 8px;
  min-width: 0;
}

.app-count {
  font-size: 12px;
  color: $text-muted;
}

.cache-label {
  font-size: 11px;
  color: $text-muted;
  opacity: 0.85;
}

.search-box {
  padding: 0 14px 12px;
  flex-shrink: 0;
}

.search-input {
  width: 100%;
  height: 34px;
  padding: 0 12px;
  border-radius: $radius-button;
  border: 1px solid $border-subtle;
  background: rgba(255, 255, 255, 0.04);
  color: $text-primary;
  font-size: 13px;
  outline: none;

  &:focus {
    border-color: rgba(124, 58, 237, 0.45);
  }

  &::placeholder {
    color: $text-muted;
  }
}

.panel-loading {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: $text-secondary;
  font-size: 13px;
}

.app-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 12px;
}

.app-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px;
  margin-bottom: 4px;
  border: 1px solid transparent;
  border-radius: $radius-button;
  background: transparent;
  cursor: pointer;
  text-align: left;
  transition: all 0.2s ease;

  &:hover:not(:disabled) {
    background: $bg-card-hover;
    border-color: $border-subtle;
  }

  &.active {
    background: rgba(124, 58, 237, 0.15);
    border-color: rgba(124, 58, 237, 0.35);
  }

  &.is-system {
    opacity: 0.45;
    cursor: not-allowed;
  }
}

.app-avatar {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: $gradient-primary;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;

  &.is-dim {
    opacity: 0.55;
  }
}

.app-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  object-fit: contain;
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.04);

  &.is-dim {
    opacity: 0.55;
  }
}

.list-divider {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 10px 8px;
  margin-top: 4px;
}

.divider-line {
  flex: 1;
  height: 1px;
  background: $border-subtle;
}

.divider-text {
  font-size: 11px;
  color: $text-muted;
  white-space: nowrap;
}

.app-meta {
  flex: 1;
  min-width: 0;
}

.app-name {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-sub {
  font-size: 11px;
  color: $text-muted;
  margin-top: 2px;
}

.system-tag {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: $text-muted;
  flex-shrink: 0;
}

.bundle-id {
  margin: 4px 0 0;
  font-size: 11px;
  color: $text-muted;
}

.related-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  text-align: center;

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
  }

  h3 {
    margin: 0 0 8px;
    color: $text-primary;
  }

  p {
    margin: 0;
    color: $text-muted;
    font-size: 13px;
    line-height: 1.6;
  }
}

.related-summary {
  padding: 0 18px 12px;
  font-size: 13px;
  color: $text-secondary;

  strong {
    color: $color-green;
    font-size: 16px;
  }
}

.related-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px;
}

.related-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);

  &:last-child {
    border-bottom: none;
  }
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-title {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
}

.category-tag {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(59, 130, 246, 0.15);
  color: $color-blue;
}

.item-path {
  font-size: 11px;
  color: $text-muted;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 4px;
}

.item-side {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  min-width: 108px;
}

.item-side-top {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-side-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

.path-action {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 6px;
  border: 1px solid $border-subtle;
  background: rgba(255, 255, 255, 0.04);
  color: $text-secondary;
  font-size: 10px;
  cursor: pointer;
  white-space: nowrap;

  &:hover {
    background: $bg-card-hover;
    color: $text-primary;
  }
}

.item-desc {
  font-size: 11px;
  color: $text-muted;
  margin-top: 2px;
}

.item-size {
  font-size: 12px;
  font-weight: 600;
  color: $text-secondary;
  flex-shrink: 0;
}

.related-actions {
  display: flex;
  gap: 10px;
  padding: 14px 18px;
  border-top: 1px solid $border-subtle;
  flex-shrink: 0;
}

.uninstall-overlay {
  position: absolute;
  inset: 0;
  z-index: 10;
  flex-direction: column;
  gap: 12px;
  background: rgba(13, 13, 18, 0.85);
  backdrop-filter: blur(6px);
  border-radius: $radius-card;
  color: $text-primary;
  font-size: 14px;
}

.flex-c-c {
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
