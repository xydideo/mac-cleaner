<template>
  <div class="disk-ring" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
      <defs>
        <linearGradient id="ringGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#7c3aed" />
          <stop offset="100%" stop-color="#3b82f6" />
        </linearGradient>
      </defs>
      <circle
        :cx="center"
        :cy="center"
        :r="radius"
        fill="none"
        stroke="rgba(255,255,255,0.08)"
        :stroke-width="strokeWidth"
      />
      <circle
        :cx="center"
        :cy="center"
        :r="radius"
        fill="none"
        stroke="url(#ringGradient)"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="dashOffset"
        :transform="`rotate(-90 ${center} ${center})`"
        class="ring-progress"
      />
    </svg>
    <div class="ring-content">
      <div class="ring-percent">{{ Math.round(percent) }}%</div>
      <div class="ring-label">{{ label }}</div>
      <div v-if="subLabel" class="ring-sublabel">{{ subLabel }}</div>
    </div>
    <div v-if="scanning" class="ring-scanning" />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"

const props = withDefaults(
  defineProps<{
    percent: number
    label?: string
    subLabel?: string
    size?: number
    scanning?: boolean
  }>(),
  {
    label: "已使用",
    size: 220,
    scanning: false,
  }
)

const strokeWidth = 12
const center = computed(() => props.size / 2)
const radius = computed(() => props.size / 2 - strokeWidth)
const circumference = computed(() => 2 * Math.PI * radius.value)
const dashOffset = computed(() => circumference.value * (1 - Math.min(props.percent, 100) / 100))
</script>

<style scoped lang="scss">
.disk-ring {
  position: relative;
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
}

.ring-progress {
  transition: stroke-dashoffset 0.8s cubic-bezier(0.22, 1, 0.36, 1);
}

.ring-content {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.ring-percent {
  font-size: 36px;
  font-weight: 700;
  color: $text-primary;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.ring-label {
  margin-top: 6px;
  font-size: 13px;
  color: $text-secondary;
}

.ring-sublabel {
  margin-top: 4px;
  font-size: 12px;
  color: $color-green;
  font-weight: 500;
}

.ring-scanning {
  position: absolute;
  inset: -8px;
  border-radius: 50%;
  border: 2px solid transparent;
  border-top-color: rgba(124, 58, 237, 0.6);
  animation: spin 1.2s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
