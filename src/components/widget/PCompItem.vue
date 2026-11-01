<script setup lang="ts">
import { pclFormatLocale, pclLocale, pclText } from '@/api/pclLocale'
import { compSourceOf } from '@/api/compProjects'
import { compLoaders, compTags, compVersions } from '@/api/compResources'
import { quickDownload } from '@/api/compDownload'
import type { CompType, ISearchHit } from '@/types/comp'
import { useCompFavorites } from '@/stores/compFavorites'
import sideTip from '@/composables/sideTip'
import { computed, ref } from 'vue'
import PSvgIcon from './PSvgIcon.vue'
import PListActions from './PListActions.vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const base = import.meta.env.BASE_URL
const favorites = useCompFavorites()
const props = withDefaults(
  defineProps<{
    clickable?: boolean
    data: ISearchHit
    gameVersion?: string
    loader?: string
    resourceType?: CompType
  }>(),
  {
    clickable: false,
  },
)
const downloading = ref(false)
const sourceName = computed(() =>
  compSourceOf(props.data.project_id) === 'curseforge' ? 'CurseForge' : 'Modrinth',
)
const tags = computed(() => compTags(props.data))
const versionText = computed(() =>
  [compLoaders(props.data), compVersions(props.data.versions)].filter(Boolean).join(' '),
)
const downloadsText = computed(() =>
  new Intl.NumberFormat(pclFormatLocale.value, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(props.data.downloads),
)
const dateText = computed(() => {
  const seconds = (new Date(props.data.date_modified).getTime() - Date.now()) / 1000
  if (!Number.isFinite(seconds)) return ''
  const unit =
    Math.abs(seconds) >= 31536000
      ? 'year'
      : Math.abs(seconds) >= 2592000
        ? 'month'
        : Math.abs(seconds) >= 86400
          ? 'day'
          : Math.abs(seconds) >= 3600
            ? 'hour'
            : 'minute'
  const divisor = { year: 31536000, month: 2592000, day: 86400, hour: 3600, minute: 60 }[unit]
  return new Intl.RelativeTimeFormat(pclLocale.value, { numeric: 'auto' }).format(
    Math.round(seconds / divisor),
    unit,
  )
})

function navigateToProject() {
  if (props.clickable)
    router.push({
      name: 'resource',
      query: {
        id: props.data.project_id,
        type: props.resourceType || (props.loader === 'datapack' ? 'datapack' : undefined),
        version: props.gameVersion || 'all',
        loader:
          props.data.project_type === 'mod' && props.loader !== 'datapack'
            ? props.loader || undefined
            : undefined,
      },
    })
}

async function download() {
  if (downloading.value) return
  downloading.value = true
  try {
    await quickDownload({
      ...props.data,
      project_type:
        props.resourceType || (props.loader === 'datapack' ? 'datapack' : props.data.project_type),
    })
  } catch (error) {
    sideTip.show(pclText('Proto.Error.Resource', [String(error)]), 'warn')
  } finally {
    downloading.value = false
  }
}
</script>

<template>
  <article class="comp-item" :class="{ clickable }">
    <button
      v-if="clickable"
      type="button"
      class="project-link"
      :aria-label="data.title"
      :title="data.description"
      @click="navigateToProject"
    />
    <i class="comp-icon"
      ><img :src="data.icon_url || `${base}pcl/images/Icons/NoIcon.png`" alt="" loading="lazy"
    /></i>
    <div class="comp-content">
      <div class="lab-title">{{ data.title }}</div>
      <div class="description-row">
        <span class="desc-tag" v-for="tag in tags" :key="tag">{{ tag }}</span>
        <span class="lab-desc" :title="data.description">{{ data.description }}</span>
      </div>
      <div class="metadata">
        <span class="inline-icon" :title="[compLoaders(data), ...(data.versions || [])].join(' ')"
          ><PSvgIcon name="lucide/layout-list" :size="12" /><span>{{ versionText }}</span></span
        >
        <span class="inline-icon"
          ><PSvgIcon name="lucide/download" :size="12" /><span>{{ downloadsText }}</span></span
        >
        <span
          class="inline-icon"
          :title="new Date(data.date_modified).toLocaleString(pclFormatLocale)"
          ><PSvgIcon name="lucide/arrow-up-from-line" :size="12" /><span>{{ dateText }}</span></span
        >
        <span class="inline-icon"
          ><PSvgIcon name="lucide/globe" :size="12" /><span>{{ sourceName }}</span></span
        >
      </div>
    </div>
    <PListActions v-if="clickable" class="comp-actions">
      <button
        class="pcl-icon-button"
        type="button"
        :disabled="downloading"
        :title="pclText('Download.Comp.QuickDownload.Btn.ToolTip')"
        :aria-label="pclText('Download.Comp.QuickDownload.Btn.ToolTip')"
        @click="download"
      >
        <PSvgIcon name="lucide/download" :size="15" />
      </button>
      <button
        class="pcl-icon-button"
        type="button"
        :title="
          pclText(
            favorites.has(data.project_id)
              ? 'Download.Comp.Detail.Unfavorite'
              : 'Download.Comp.Detail.Favorite',
          )
        "
        :aria-label="
          pclText(
            favorites.has(data.project_id)
              ? 'Download.Comp.Detail.Unfavorite'
              : 'Download.Comp.Detail.Favorite',
          )
        "
        :aria-pressed="favorites.has(data.project_id)"
        @click="favorites.toggle(data)"
      >
        <PSvgIcon
          :name="favorites.has(data.project_id) ? 'lucide/heart-filled' : 'lucide/heart'"
          :size="15"
        />
      </button>
    </PListActions>
  </article>
</template>

<style scoped>
.comp-item {
  position: relative;
  display: flex;
  gap: 8px;
  height: 64px;
  border-radius: 4px;
  padding: 5px 5px 5px 7px;
  align-items: center;
  color: var(--color-text-grey);
  /* CE MyListItem：移出 180 ms、指向 120 ms。 */
  transition: background-color 180ms linear;
  min-width: 0;
}
.comp-item.clickable:hover,
.comp-item.clickable:focus-within {
  background-color: var(--color-tint-lighter);
  transition-duration: 120ms;
}
.project-link {
  position: absolute;
  inset: 0;
  width: 100%;
  border: 0;
  border-radius: inherit;
  background: transparent;
  cursor: pointer;
}
.comp-icon {
  flex-shrink: 0;
  width: 50px;
  height: 50px;
}
.comp-icon img {
  width: 50px;
  height: 50px;
  border-radius: 6px;
}
.comp-content {
  flex: 1;
  min-width: 0;
}
.lab-title {
  font-size: 14px;
  line-height: 20px;
  color: var(--color-text-black);
}
.lab-title,
.lab-desc {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.description-row {
  display: flex;
  align-items: center;
  height: 18px;
  font-size: 12px;
  gap: 4px;
  overflow: hidden;
}
.lab-desc {
  position: relative;
  pointer-events: none;
}
.desc-tag {
  flex-shrink: 0;
  border-radius: 3px;
  font-size: 11px;
  line-height: 15px;
  background: rgba(140, 140, 140, 0.15);
  color: var(--color-text-grey);
  padding: 0 3px;
}
.metadata {
  display: grid;
  grid-template-columns: minmax(0, 1.65fr) minmax(0, 0.65fr) minmax(0, 0.8fr) auto;
  align-items: center;
  gap: 10px;
  height: 16px;
  font-size: 12px;
  line-height: 16px;
}
.inline-icon {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
  min-width: 0;
}
.inline-icon > span {
  overflow: hidden;
  text-overflow: ellipsis;
}
.comp-actions {
  position: relative;
  flex-shrink: 0;
}
@media (max-width: 600px) {
  .metadata {
    gap: 6px;
    grid-template-columns: minmax(0, 1.65fr) minmax(0, 0.65fr) minmax(0, 0.8fr);
  }
  .inline-icon:last-child {
    display: none;
  }
}
</style>
