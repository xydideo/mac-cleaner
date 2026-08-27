<template>
  <div class="segmented-control segmented-control--compact" role="tablist">
    <button
      type="button"
      class="segmented-control__item"
      :class="{ 'is-active': modelValue === 'folders' }"
      :disabled="disabled"
      role="tab"
      :aria-selected="modelValue === 'folders'"
      @click="select('folders')"
    >
      大文件夹
    </button>
    <button
      type="button"
      class="segmented-control__item"
      :class="{ 'is-active': modelValue === 'files' }"
      :disabled="disabled"
      role="tab"
      :aria-selected="modelValue === 'files'"
      @click="select('files')"
    >
      大文件
    </button>
  </div>
</template>

<script setup lang="ts">
import type { LargeScanKind } from "@/api/folder"

defineProps<{
  modelValue: LargeScanKind
  disabled?: boolean
}>()

const emit = defineEmits<{
  "update:modelValue": [value: LargeScanKind]
  change: [value: LargeScanKind]
}>()

const select = (value: LargeScanKind) => {
  emit("update:modelValue", value)
  emit("change", value)
}
</script>
