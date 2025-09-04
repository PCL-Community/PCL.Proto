<script setup lang="ts">
import { computed } from 'vue'
import IconDownload from '../icons/header/IconDownload.vue'
import IconSetup from '../icons/header/IconSetup.vue'
import IconWeb from '@/assets/icons/IconWeb.svg'
import IconTimeUp from '@/assets/icons/IconTimeUp.svg'
import type { ISearchHit } from '@/api/modrinthApi'
import { useRouter } from 'vue-router'
const router = useRouter()

const props = withDefaults(defineProps<{ clickable?: boolean; data: ISearchHit }>(), {
  clickable: false,
})

const downloadsText = computed(() => {
  if (props.data.downloads >= 10000) {
    return `${Math.floor(props.data.downloads / 10000)}万`
  }
  return props.data.downloads.toString()
})

const dateText = computed(() => {
  let date = new Date(props.data.date_modified)
  return date.toLocaleDateString()
})

const navigateToProject = () => {
  router.push({ name: 'resouce', query: { id: props.data.project_id } })
}
</script>

<template>
  <li class="comp-item" @click="navigateToProject" :class="{ clickable }">
    <i class="comp-icon"><img :src="data.icon_url" /></i>
    <div class="lab-title-row">
      <span class="lab-title">{{ data.title }}</span
      >&nbsp;&nbsp;|&nbsp;&nbsp;<span>{{ data.title }}</span>
    </div>
    <div class="lab-desc" :title="data.description">
      <span class="desc-tag" v-for="category in data.categories">{{ category }}</span
      >{{ data.description }}
    </div>
    <p class="inline-icon"><IconSetup />{{ data.categories[0] }}</p>
    <p class="inline-icon icon-download"><IconDownload />{{ downloadsText }}</p>
    <p class="inline-icon"><IconTimeUp />{{ dateText }}</p>
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
  transition:
    background-color 0.4s,
    scale 0.2s;
  align-items: center;
  line-height: 1rem;
  color: var(--color-text-grey);
  overflow: hidden;
}

.comp-item.clickable:hover {
  background-color: var(--color-tint-lighter);
}

.comp-item.clickable:active {
  scale: 0.98;
  background-color: var(--half-transparent-blue);
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

:deep(.inline-icon > svg) {
  width: 13px;
  height: 13px;
  max-width: 13px !important;
  max-height: 13px !important;
}

:deep(.icon-download > svg) {
  width: 11px;
}

.desc-tag {
  display: inline-block;
  border-radius: 4px;
  font-size: 11px;
  background-color: gainsboro;
  padding: 0 3px;
  margin-inline-end: 4px;
}
</style>
