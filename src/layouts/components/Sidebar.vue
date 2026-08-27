<template>
  <nav class="sidebar">
    <div class="sidebar-logo">
      <img src="/image.png" :alt="APP_NAME_CN" class="logo-icon" />
    </div>

    <router-link
      v-for="item in sidebarNavItems"
      :key="item.path"
      :to="item.path"
      class="nav-item"
      :class="{ active: isActive(item.path) }"
      :title="item.title"
    >
      <el-icon :size="22">
        <component :is="item.icon" />
      </el-icon>
      <span class="nav-label">{{ item.title }}</span>
    </router-link>

    <div class="sidebar-spacer" />

    <router-link
      :to="settingsNavItem.path"
      class="nav-item"
      :class="{ active: isActive(settingsNavItem.path) }"
      :title="settingsNavItem.title"
    >
      <el-icon :size="22"><component :is="settingsNavItem.icon" /></el-icon>
      <span class="nav-label">{{ settingsNavItem.title }}</span>
    </router-link>
  </nav>
</template>

<script setup lang="ts">
import { useRoute } from "vue-router"
import { APP_NAME_CN } from "@/constants/app"
import { settingsNavItem, sidebarNavItems } from "@/constants/navigation"

const route = useRoute()

const isActive = (path: string) => {
  if (path === "/") return route.path === "/"
  return route.path.startsWith(path)
}
</script>

<style scoped lang="scss">
.sidebar {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  width: $sidebar-width;
  padding: 16px 0;
  background: $bg-secondary;
  border-right: 1px solid $border-subtle;
  flex-shrink: 0;
}

.sidebar-logo {
  margin-bottom: 20px;
}

.logo-icon {
  display: block;
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 56px;
  padding: 10px 0;
  margin-bottom: 4px;
  border-radius: 12px;
  color: $text-muted;
  transition: all 0.2s ease;
  position: relative;

  &:hover {
    color: $text-secondary;
    background: $bg-card-hover;
  }

  &.active {
    color: #fff;
    background: rgba(124, 58, 237, 0.2);

    &::before {
      content: "";
      position: absolute;
      left: -1px;
      top: 50%;
      transform: translateY(-50%);
      width: 3px;
      height: 24px;
      border-radius: 0 3px 3px 0;
      background: $gradient-primary;
    }
  }
}

.nav-label {
  font-size: 10px;
  font-weight: 500;
}

.sidebar-spacer {
  flex: 1;
}
</style>
