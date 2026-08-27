<template>
  <header class="app-header" data-tauri-drag-region>
    <div class="header-left">
      <WindowTrafficLights />
      <div class="brand">
        <span class="app-title">{{ APP_NAME_CN }}</span>
        <span class="app-subtitle">{{ APP_NAME }}</span>
      </div>
    </div>
    <div class="header-center" data-tauri-drag-region>
      <span class="page-title">{{ currentTitle }}</span>
    </div>
    <div class="header-right" />
  </header>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { useRoute } from "vue-router"
import { APP_NAME, APP_NAME_CN } from "@/constants/app"
import WindowTrafficLights from "./WindowTrafficLights.vue"

const route = useRoute()
const currentTitle = computed(() => (route.meta.title as string) || APP_NAME_CN)
</script>

<style scoped lang="scss">
.app-header {
  display: flex;
  align-items: center;
  height: $header-height;
  padding: 0 20px 0 16px;
  flex-shrink: 0;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 200px;
}

.brand {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.app-title {
  font-size: 13px;
  font-weight: 600;
  color: $text-primary;
  letter-spacing: 0.02em;
  line-height: 1.2;
}

.app-subtitle {
  font-size: 10px;
  font-weight: 500;
  color: $text-muted;
  letter-spacing: 0.04em;
  text-transform: lowercase;
}

.header-center {
  flex: 1;
  text-align: center;
}

.page-title {
  font-size: 14px;
  font-weight: 600;
  color: $text-primary;
}

.header-right {
  min-width: 200px;
}
</style>
