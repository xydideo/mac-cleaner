<template>
  <div v-if="!isDesktop" class="desktop-only">
    <div class="desktop-only-card">
      <img src="/image.png" :alt="APP_NAME_CN" class="desktop-only-logo" />
      <h1>{{ APP_NAME_CN }}</h1>
      <p>这是 Mac 桌面应用，不支持在浏览器中打开。</p>
      <p class="hint">请在终端运行：<code>pnpm dev:t</code></p>
    </div>
  </div>
  <MainLayout v-else />
</template>

<script setup lang="ts">
import MainLayout from "./layouts/MainLayout.vue"
import { APP_NAME_CN } from "@/constants/app"
import { isTauriRuntime } from "@/utils/tauriPlatform"

const isDesktop = isTauriRuntime()
</script>

<style scoped lang="scss">
.desktop-only {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $bg-primary;
  color: $text-primary;
}

.desktop-only-card {
  text-align: center;
  padding: 48px 56px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid $border-subtle;
  max-width: 420px;
}

.desktop-only-logo {
  width: 72px;
  height: 72px;
  margin-bottom: 16px;
}

h1 {
  font-size: 24px;
  margin-bottom: 12px;
}

p {
  color: $text-secondary;
  line-height: 1.6;
}

.hint {
  margin-top: 20px;
  font-size: 14px;
}

code {
  padding: 2px 8px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.08);
  color: $color-purple;
}
</style>
