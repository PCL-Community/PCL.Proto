<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { pclFormatLocale, pclText as text } from '@/api/pclLocale'
import { downloadHomepageText, homepageRevision } from '@/api/customHomepage'
import { parseHomepageNews, type HomepageNewsItem } from '@/util/homepageNews'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'

defineProps<{ embedded?: boolean }>()
const url =
  'https://net-secondary.web.minecraft-services.net/api/v1.0/zh-cn/search?pageSize=24&sortType=Recent&category=News&newsOnly=true&page=1'
const items = ref<HomepageNewsItem[]>([])
const loading = ref(false)
const error = ref<string>()
let controller: AbortController | undefined
let generation = 0
watch(
  homepageRevision,
  async () => {
    controller?.abort()
    controller = new AbortController()
    const signal = controller.signal
    const request = ++generation
    loading.value = true
    error.value = undefined
    try {
      const result = parseHomepageNews(await downloadHomepageText(url, signal))
      if (request === generation) items.value = result
    } catch (failure) {
      if (request === generation && !signal.aborted) error.value = String(failure)
    } finally {
      if (request === generation) loading.value = false
    }
  },
  { immediate: true },
)
onBeforeUnmount(() => {
  generation++
  controller?.abort()
})

function decodeText(value: string) {
  const textarea = document.createElement('textarea')
  textarea.innerHTML = value
  return textarea.value
}
function date(value?: number) {
  return value === undefined ? '' : new Date(value * 1000).toLocaleDateString(pclFormatLocale.value)
}
</script>
<template>
  <section :class="{ 'pcl-page': !embedded }" class="news-page">
    <PLoading
      v-if="loading && !items.length"
      :loading-text="text('Launch.Homepage.News.Loading')"
    />
    <PHint v-if="error" severity="error">{{ error }}</PHint>
    <div class="news-grid">
      <PCard v-for="(item, index) in items" :key="item.url || `${item.title}-${index}`" hide-title>
        <div class="news-content">
          <img
            v-if="item.image"
            :src="item.image"
            :alt="item.imageAltText"
            loading="lazy"
            @error="($event.target as HTMLImageElement).hidden = true"
            @load="($event.target as HTMLImageElement).hidden = false"
          />
          <strong>{{ item.title }}</strong>
          <div class="news-meta">
            <span>{{ item.author }}</span
            ><span>{{ date(item.time) }}</span>
          </div>
          <p>{{ decodeText(item.description) }}</p>
          <a v-if="item.url" :href="item.url" target="_blank" rel="noopener noreferrer">{{
            text('Launch.Homepage.News.ViewDetail')
          }}</a>
        </div>
      </PCard>
    </div>
    <p v-if="!loading && !error && !items.length" class="pcl-empty">
      {{ text('Proto.Empty.News') }}
    </p>
  </section>
</template>
<style scoped>
.news-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 200px), 1fr));
  gap: 14px;
  align-items: start;
}
.news-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  min-width: 0;
}
.news-content img {
  width: 100%;
  aspect-ratio: 2.2;
  object-fit: cover;
  border-radius: 6px;
}
.news-content strong {
  font-size: 16px;
  font-weight: 600;
  overflow-wrap: anywhere;
}
.news-meta {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  color: var(--color-text-grey);
  font-size: 12px;
}
.news-content p {
  white-space: pre-line;
  overflow-wrap: anywhere;
  color: var(--color-text-grey);
}
.news-content a {
  align-self: flex-end;
}
</style>
