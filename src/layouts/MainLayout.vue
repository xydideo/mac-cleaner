<template>
  <div class="main-layout">
    <div class="window-bg" />
    <AppSidebar />
    <div class="main-area">
      <AppHeader />
      <main class="main-content">
        <router-view v-slot="{ Component, route }">
          <Transition name="page-fade" mode="out-in">
            <KeepAlive :include="cachedViews">
              <component :is="Component" :key="route.name" />
            </KeepAlive>
          </Transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppHeader from "./components/AppHeader.vue"
import AppSidebar from "./components/Sidebar.vue"

const cachedViews = ["AppUninstall", "CustomFolder"]
</script>

<style scoped lang="scss">
.main-layout {
  position: relative;
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.window-bg {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 20% 0%, rgba(124, 58, 237, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 100%, rgba(59, 130, 246, 0.08) 0%, transparent 50%),
    $bg-primary;
  z-index: 0;
}

.main-area {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.main-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 28px 28px;
}
</style>
