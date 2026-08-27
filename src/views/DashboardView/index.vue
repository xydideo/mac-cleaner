<template>
  <div class="dashboard">
    <div class="hero-section">
      <div class="hero-ring">
        <DiskRing
          :percent="diskInfo?.usage_percent ?? 0"
          :sub-label="reclaimableLabel"
          :scanning="scanning"
        />
      </div>

      <div class="hero-actions">
        <el-tooltip placement="top" popper-class="scan-scope-tooltip">
          <template #content>
            <div class="scan-scope-tip">{{ QUICK_SCAN_SCOPE_TIP }}</div>
          </template>
          <button class="gradient-btn" :disabled="scanning" @click="handleSmartScan">
            <el-icon v-if="scanning" class="is-loading"><Loading /></el-icon>
            {{ scanning ? "扫描中..." : "智能扫描" }}
          </button>
        </el-tooltip>
        <el-tooltip placement="top" popper-class="scan-scope-tooltip">
          <template #content>
            <div class="scan-scope-tip">{{ DEEP_SCAN_SCOPE_TIP }}</div>
          </template>
          <button class="secondary-btn" :disabled="scanning" @click="handleDeepScan">
            深度扫描
          </button>
        </el-tooltip>
      </div>

      <div v-if="scanProgress && scanning" class="scan-status glass-card">
        <el-progress :percentage="Math.round(scanProgress.percent)" :show-text="false" />
        <span class="scan-path">{{ scanPaused ? "扫描已暂停" : scanProgress.current_path }}</span>
        <ScanControlButtons :paused="scanPaused" @pause="pauseScan" @resume="resumeScan" />
      </div>
    </div>

    <div class="stats-row">
      <div class="stat-card glass-card">
        <div class="stat-label">总容量</div>
        <div class="stat-value">{{ formatSize(diskInfo?.total_bytes ?? 0) }}</div>
      </div>
      <div class="stat-card glass-card">
        <div class="stat-label">已使用</div>
        <div class="stat-value is-used">{{ formatSize(diskInfo?.used_bytes ?? 0) }}</div>
      </div>
      <div class="stat-card glass-card">
        <div class="stat-label">可用空间</div>
        <div class="stat-value is-free">{{ formatSize(diskInfo?.available_bytes ?? 0) }}</div>
      </div>
    </div>

    <div class="category-grid">
      <CategoryCard
        icon="🟣"
        icon-bg="rgba(124,58,237,0.2)"
        title="应用缓存"
        size="扫描后显示"
        desc="Library/Caches"
        @click="goScan('/junk')"
      />
      <CategoryCard
        icon="🟡"
        icon-bg="rgba(245,158,11,0.2)"
        title="系统日志"
        size="扫描后显示"
        desc="Library/Logs"
        @click="goScan('/junk')"
      />
      <CategoryCard
        icon="🔵"
        icon-bg="rgba(59,130,246,0.2)"
        title="系统遗留"
        size="扫描后显示"
        desc="plist / 启动项"
        @click="goScan('/junk')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue"
import { useRouter } from "vue-router"
import { Loading } from "@element-plus/icons-vue"
import DiskRing from "@/components/DiskRing.vue"
import ScanControlButtons from "@/components/ScanControlButtons.vue"
import CategoryCard from "@/components/CategoryCard.vue"
import { useDiskInfo, useScanner } from "@/composables/useScanner"
import { QUICK_SCAN_SCOPE_TIP, DEEP_SCAN_SCOPE_TIP } from "@/constants/scanScopeTips"
import { formatSize } from "@/utils/formatSize"

const router = useRouter()
const { diskInfo, load } = useDiskInfo()
const { scanning, scanPaused, scanProgress, scan, pauseScan, resumeScan, teardownListeners } = useScanner()

const reclaimableLabel = computed(() => {
  if (!diskInfo.value) return undefined
  const free = formatSize(diskInfo.value.available_bytes)
  return `可用 ${free}`
})

onMounted(() => load())
onUnmounted(() => teardownListeners())

const handleSmartScan = async () => {
  await scan("quick")
  router.push("/junk")
}

const handleDeepScan = async () => {
  await scan("deep")
  router.push("/junk")
}

const goScan = (path: string) => {
  router.push(path)
}
</script>

<style scoped lang="scss">
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 28px;
  padding-top: 8px;
}

.hero-section {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 20px;
  padding: 8px 0;
}

.hero-ring {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 0;
}

.hero-actions {
  display: flex;
  justify-content: center;
  gap: 12px;
}

.secondary-btn {
  padding: 14px 24px;
  border-radius: $radius-button;
  border: 1px solid $border-subtle;
  background: $bg-card;
  color: $text-secondary;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover:not(:disabled) {
    border-color: rgba(124, 58, 237, 0.4);
    color: $text-primary;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.scan-status {
  width: 100%;
  max-width: 420px;
  margin: 0 auto;
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.scan-path {
  font-size: 12px;
  color: $text-muted;
  text-align: center;
}

.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
}

.stat-card {
  padding: 18px 20px;
}

.stat-label {
  font-size: 12px;
  color: $text-muted;
  margin-bottom: 6px;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: $text-primary;

  &.is-used { color: $color-purple; }
  &.is-free { color: $color-green; }
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
}
</style>
