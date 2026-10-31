<script setup lang="ts">
import PListItem from './PListItem.vue'
import PSvgIcon from './PSvgIcon.vue'
import { pclText } from '@/api/pclLocale'
defineProps<{
  id: string
  title: string
  description?: string
  icon?: string
  enabled?: boolean
  updatable?: boolean
  error?: string
  selected?: boolean
}>()
const emit = defineEmits<{ select: [boolean]; toggle: []; update: [] }>()
const fallback = `${import.meta.env.BASE_URL}pcl/images/Icons/NoIcon.png`
</script>
<template>
  <PListItem
    class="local-resource"
    :title="title"
    :subtitle="error || description"
    :class="{ disabled: enabled === false }"
  >
    <template #icon
      ><input
        type="checkbox"
        :aria-label="pclText('Proto.Label.Select', [title])"
        :checked="selected"
        @change="emit('select', ($event.target as HTMLInputElement).checked)" /><img
        :src="icon || fallback"
        alt=""
        @error="($event.target as HTMLImageElement).src = fallback"
    /></template>
    <template #icon-actions
      ><button
        v-if="updatable"
        class="pcl-icon-button"
        type="button"
        @click="() => emit('update')"
        :title="pclText('Instance.Resource.Filter.Updatable')"
        :aria-label="pclText('Instance.Resource.Filter.Updatable')"
      >
        <PSvgIcon name="lucide/upload" :size="15" /></button
      ><button
        class="pcl-icon-button"
        type="button"
        @click="() => emit('toggle')"
        :title="
          pclText(
            enabled === false
              ? 'Instance.Resource.Filter.Enabled'
              : 'Instance.Resource.Filter.Disabled',
          )
        "
        :aria-label="
          pclText(
            enabled === false
              ? 'Instance.Resource.Filter.Enabled'
              : 'Instance.Resource.Filter.Disabled',
          )
        "
      >
        <PSvgIcon :name="enabled === false ? 'lucide/check' : 'lucide/x'" :size="15" /></button
    ></template>
    <template #actions><slot name="actions" /></template>
  </PListItem>
</template>
<style scoped>
.local-resource {
  min-height: 44px;
  --list-title-size: 14px;
}
.local-resource img {
  width: 34px;
  height: 34px;
  border-radius: 6px;
}
.disabled {
  opacity: 0.5;
}
</style>
