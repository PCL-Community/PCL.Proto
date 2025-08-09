<script setup lang="ts">
import { computed } from 'vue'
import IconDownload from '../icons/header/IconDownload.vue'
import IconSetup from '../icons/header/IconSetup.vue'
import IconWeb from '@/assets/icons/IconWeb.svg'

const props = defineProps<{
  title: string
  description: string
  icon_url: string
  categories: string[]
  downloads: number
  date_modified: string
}>()

const downloadsText = computed(() => {
  if (props.downloads >= 10000) {
    return `${Math.floor(props.downloads / 10000)}万`
  }
  return props.downloads.toString()
})

const dateText = computed(() => {
  let date = new Date(props.date_modified)
  return date.toLocaleDateString()
})
</script>

<template>
  <li class="comp-item">
    <i class="comp-icon"><img :src="icon_url" /></i>
    <div class="lab-title-row">
      <span class="lab-title">{{ title }}</span
      >&nbsp;&nbsp;|&nbsp;&nbsp;<span>{{ title }}</span>
    </div>
    <div class="lab-desc">{{ description }}</div>
    <p class="inline-icon"><IconSetup />{{ categories[0] }}</p>
    <p class="inline-icon icon-download"><IconDownload />{{ downloadsText }}</p>
    <p class="inline-icon"><IconSetup />{{ dateText }}</p>
    <p class="inline-icon"><IconWeb />Modrinth</p>
  </li>
</template>

<style lang="css" scoped>
.comp-item {
  display: grid;
  grid-template-columns: 50px 2fr 1fr 1fr 1fr;
  grid-template-rows: 5fr 4fr 4fr;
  gap: 0 10px;
  border-radius: 4px;
  padding: 6px;
  transition: background-color 0.4s;
  align-items: center;
  line-height: 1rem;
  color: var(--color-text-grey);
  overflow: hidden;
}

.comp-item:hover {
  background-color: var(--color-tint-lighter);
}

.comp-icon {
  grid-row: 1 / 4;
  grid-column: 1 / 2;
}

.lab-title-row {
  grid-column: 2 / 6;
}

.comp-icon img {
  height: 50px;
  width: 50px;
  border-radius: 6px;
}

.lab-title {
  font-size: 15px;
  color: var(--color-text-black);
}

.lab-desc {
  grid-column: 2 / 6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inline-icon {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inline-icon > svg {
  max-width: 13px !important;
  max-height: 13px !important;
}

.icon-download > svg {
  width: 11px;
}
</style>
