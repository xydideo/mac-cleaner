<template>
  <div class="category-card glass-card" @click="$emit('click')">
    <div class="card-icon" :style="{ background: iconBg }">{{ icon }}</div>
    <div class="card-body">
      <div class="card-title">{{ title }}</div>
      <div class="card-size">{{ size }}</div>
      <div v-if="desc" class="card-desc">{{ desc }}</div>
    </div>
    <el-icon class="card-arrow"><ArrowRight /></el-icon>
  </div>
</template>

<script setup lang="ts">
import { ArrowRight } from "@element-plus/icons-vue"

defineProps<{
  icon: string
  iconBg?: string
  title: string
  size: string
  desc?: string
}>()

defineEmits<{ click: [] }>()
</script>

<style scoped lang="scss">
.category-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 18px;
  cursor: pointer;
  transition: background 0.2s ease, transform 0.2s ease, border-color 0.2s ease;

  &:hover {
    background: $bg-card-hover;
    border-color: rgba(124, 58, 237, 0.25);
    transform: translateY(-2px);
  }
}

.card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: 12px;
  font-size: 20px;
  flex-shrink: 0;
}

.card-body {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: $text-primary;
}

.card-size {
  margin-top: 2px;
  font-size: 16px;
  font-weight: 700;
  color: $text-primary;
}

.card-desc {
  margin-top: 2px;
  font-size: 12px;
  color: $text-muted;
}

.card-arrow {
  color: $text-muted;
  flex-shrink: 0;
}
</style>
