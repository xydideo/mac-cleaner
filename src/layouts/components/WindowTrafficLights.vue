<template>
  <div
    v-if="visible"
    class="traffic-lights"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
    @dblclick.stop
  >
    <button class="traffic-light is-close" type="button" title="关闭" @click.stop="handleClose">
      <svg v-show="hovered" class="traffic-light__icon" viewBox="0 0 12 12">
        <path d="M3.2 3.2 L8.8 8.8 M8.8 3.2 L3.2 8.8" />
      </svg>
    </button>
    <button class="traffic-light is-minimize" type="button" title="最小化" @click.stop="handleMinimize">
      <svg v-show="hovered" class="traffic-light__icon" viewBox="0 0 12 12">
        <path d="M2.5 6 H9.5" />
      </svg>
    </button>
    <button class="traffic-light is-maximize" type="button" :title="windowMaximized ? '还原' : '最大化'" @click.stop="handleMaximize">
      <svg v-show="hovered" class="traffic-light__icon" viewBox="0 0 12 12">
        <path v-if="windowMaximized" d="M4 3.5 H8.5 V8 M3.5 4.5 H7.5 V9" />
        <path v-else d="M6 2.8 V9.2 M2.8 6 H9.2" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { isTauriRuntime } from "@/utils/tauriPlatform"
import { ensureWindowMaximizeListener, toggleWindowMaximized, windowMaximized } from "@/utils/windowChrome"

const visible = isTauriRuntime()
const hovered = ref(false)

const handleClose = async () => {
  await getCurrentWindow().close()
}

const handleMinimize = async () => {
  await getCurrentWindow().minimize()
}

const handleMaximize = async () => {
  await toggleWindowMaximized()
}

onMounted(() => {
  if (visible) ensureWindowMaximizeListener()
})
</script>

<style scoped lang="scss">
.traffic-lights {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.traffic-light {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 12px;
  height: 12px;
  padding: 0;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  transition: filter 0.15s ease, transform 0.15s ease;

  &:hover {
    filter: brightness(1.08);
    transform: scale(1.06);
  }

  &.is-close { background-color: #ff5f57; }
  &.is-minimize { background-color: #febc2e; }
  &.is-maximize { background-color: #28c840; }
}

.traffic-light__icon {
  width: 8px;
  height: 8px;

  path {
    fill: none;
    stroke: rgba(0, 0, 0, 0.55);
    stroke-width: 1.2;
    stroke-linecap: round;
  }
}
</style>
