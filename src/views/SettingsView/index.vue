<template>
  <div class="settings">
    <div class="settings-section glass-card">
      <h3>权限</h3>
      <p class="section-desc">
        部分目录（如 Library/Caches）需要「完全磁盘访问权限」才能完整扫描。
      </p>
      <button class="pill-btn" @click="openPrivacySettings">打开系统隐私设置</button>
    </div>

    <div class="settings-section glass-card">
      <h3>清理偏好</h3>
      <div class="setting-row">
        <span>默认删除方式</span>
        <el-radio-group v-model="settings.deleteMode" size="small" class="setting-radio-group">
          <el-radio value="trash">移到废纸篓（推荐）</el-radio>
          <el-radio value="permanent">永久删除</el-radio>
        </el-radio-group>
      </div>
      <div class="setting-row">
        <span>「长期未动」阈值</span>
        <el-select v-model="settings.staleDays" size="small" class="setting-select" popper-class="dark-select-popper">
          <el-option :value="30" label="30 天" />
          <el-option :value="90" label="90 天" />
          <el-option :value="180" label="180 天" />
        </el-select>
      </div>

      <div
        ref="largeScanSettingRef"
        class="large-scan-settings"
        :class="{ 'is-focused': focusLargeScan }"
      >
        <div class="setting-row">
          <span>大文件阈值</span>
          <el-select
            v-model="settings.largeFileMB"
            size="small"
            class="setting-select"
            popper-class="dark-select-popper"
          >
            <el-option v-for="mb in FILE_MB_OPTIONS" :key="mb" :value="mb" :label="`${mb} MB`" />
          </el-select>
        </div>
        <div class="setting-row">
          <span>大文件夹阈值</span>
          <el-select
            v-model="settings.largeFolderMB"
            size="small"
            class="setting-select"
            popper-class="dark-select-popper"
          >
            <el-option
              v-for="mb in FOLDER_MB_OPTIONS"
              :key="mb"
              :value="mb"
              :label="formatThresholdMB(mb)"
            />
          </el-select>
        </div>
        <p class="setting-hint">全盘查找大文件：文件与文件夹分别按以上阈值筛选。</p>
      </div>
    </div>

    <div class="settings-section glass-card">
      <h3>关于</h3>
      <p class="section-desc">{{ APP_NAME_CN }} {{ APP_NAME }} · design by xydideo</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue"
import { useRoute } from "vue-router"
import { invoke } from "@tauri-apps/api/core"
import { ElMessage } from "element-plus"
import { formatThresholdMB, useAppSettings } from "@/composables/useAppSettings"
import { APP_NAME, APP_NAME_CN } from "@/constants/app"
import { isTauriRuntime } from "@/utils/tauriPlatform"

const route = useRoute()
const { settings, FILE_MB_OPTIONS, FOLDER_MB_OPTIONS } = useAppSettings()
const largeScanSettingRef = ref<HTMLElement | null>(null)

const focusLargeScan = computed(
  () => route.query.focus === "largeFile" || route.query.focus === "largeScan"
)

onMounted(async () => {
  if (!focusLargeScan.value) return
  await nextTick()
  largeScanSettingRef.value?.scrollIntoView({ behavior: "smooth", block: "center" })
})

const openPrivacySettings = async () => {
  if (!isTauriRuntime()) {
    ElMessage.warning("请在桌面应用中打开")
    return
  }
  try {
    await invoke("open_privacy_settings")
  } catch (e) {
    ElMessage.error(String(e))
  }
}
</script>

<style scoped lang="scss">
.settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
  max-width: 640px;
}

.settings-section {
  padding: 20px 24px;

  h3 {
    margin: 0 0 8px;
    font-size: 15px;
    font-weight: 600;
    color: $text-primary;
  }
}

.section-desc {
  margin: 0 0 14px;
  font-size: 13px;
  color: $text-muted;
  line-height: 1.6;
}

.large-scan-settings {
  border-radius: $radius-button;
  transition: background 0.2s ease, box-shadow 0.2s ease;

  &.is-focused {
    margin: 0 -10px;
    padding: 0 10px 4px;
    background: rgba(124, 58, 237, 0.1);
    box-shadow: 0 0 0 1px rgba(124, 58, 237, 0.28);
  }
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-bottom: 1px solid $border-subtle;
  color: $text-secondary;
  font-size: 13px;

  &:last-child { border-bottom: none; }
}

.setting-hint {
  margin: 4px 0 0;
  font-size: 11px;
  color: $text-muted;
}

.setting-select {
  width: 128px;
  flex-shrink: 0;
}

.setting-radio-group {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px 16px;
}
</style>
