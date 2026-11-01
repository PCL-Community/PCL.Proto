<script setup lang="ts">
import PListItem from './PListItem.vue'
import PSvgIcon from './PSvgIcon.vue'
import { pclText } from '@/api/pclLocale'
defineProps<{
  id: string
  title: string
  description?: string
  icon?: string
  version?: string
  enabled?: boolean
  togglable?: boolean
  updatable?: boolean
  error?: string
  selected?: boolean
}>()
const emit = defineEmits<{
  select: [boolean]
  toggle: []
  update: []
  info: []
  open: []
  remove: []
}>()
const fallback = `${import.meta.env.BASE_URL}pcl/images/Icons/NoIcon.png`
</script>
<template>
  <PListItem
    class="local-resource"
    :title="title"
    :subtitle="error || description"
    :class="{ disabled: enabled === false }"
    :selected="selected"
    clickable
    @click="emit('select', !selected)"
  >
    <template #title
      ><span>{{ title }}</span
      ><span v-if="version" class="resource-version"> | {{ version }}</span></template
    >
    <template #icon
      ><img
        :src="icon || fallback"
        alt=""
        @error="($event.target as HTMLImageElement).src = fallback"
    /></template>
    <template #icon-actions>
      <button
        class="pcl-icon-button"
        type="button"
        :title="pclText('Instance.Saves.Detail')"
        :aria-label="pclText('Instance.Saves.Detail')"
        @click="emit('info')"
      >
        <PSvgIcon name="lucide/info" :size="15" />
      </button>
      <button
        class="pcl-icon-button"
        type="button"
        :title="pclText('Common.Action.OpenFolder')"
        :aria-label="pclText('Common.Action.OpenFolder')"
        @click="emit('open')"
      >
        <PSvgIcon name="lucide/folder-open" :size="15" />
      </button>
      <button
        v-if="updatable"
        class="pcl-icon-button"
        type="button"
        @click="() => emit('update')"
        :title="pclText('Instance.Resource.Filter.Updatable')"
        :aria-label="pclText('Instance.Resource.Filter.Updatable')"
      >
        <PSvgIcon name="lucide/upload" :size="15" /></button
      ><button
        v-if="togglable"
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
        <PSvgIcon
          :name="enabled === false ? 'lucide/circle-check' : 'lucide/circle-minus'"
          :size="15"
        />
      </button>
      <button
        class="pcl-icon-button"
        type="button"
        :title="pclText('Common.Action.Delete')"
        :aria-label="pclText('Common.Action.Delete')"
        @click="emit('remove')"
      >
        <PSvgIcon name="lucide/trash-2" :size="15" />
      </button>
    </template>
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
  color: var(--color-text-grey);
}
.disabled :deep(.title) {
  text-decoration: line-through;
}
.local-resource[aria-pressed='true'] :deep(.title) {
  color: var(--color-tint);
}
.resource-version {
  color: var(--color-text-grey);
  font-size: 12px;
}
</style>
