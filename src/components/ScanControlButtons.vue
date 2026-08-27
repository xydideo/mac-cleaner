<template>
  <div class="scan-control-buttons">
    <button
      v-if="!paused"
      type="button"
      class="action-btn action-btn--ghost"
      :disabled="disabled"
      @click="$emit('pause')"
    >
      暂停
    </button>
    <button
      v-else
      type="button"
      class="action-btn action-btn--primary"
      :disabled="disabled"
      @click="$emit('resume')"
    >
      继续
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  paused: boolean
  disabled?: boolean
}>()

defineEmits<{
  pause: []
  resume: []
}>()
</script>

<style scoped lang="scss">
.scan-control-buttons {
  display: inline-flex;
  gap: 8px;
}
</style>
