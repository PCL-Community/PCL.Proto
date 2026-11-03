<script setup lang="ts">
import { pclText as text } from '@/api/pclLocale'
import PCard from '@/components/widget/PCard.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PHint from '@/components/widget/PHint.vue'
withDefaults(
  defineProps<{
    items?: {
      id: string
      title: string
      author: string
      publishDate: string
      description: string
      url?: string
    }[]
    loading?: boolean
    error?: string
  }>(),
  { items: () => [], loading: false, error: '' },
)
</script>
<template>
  <section class="pcl-page news-page">
    <PLoading v-if="loading" :loading-text="text('Launch.Homepage.News.Loading')" />
    <PHint v-else-if="error" severity="error">{{ error }}</PHint
    ><PCard v-for="item in items" :key="item.id" :title="item.title"
      ><div class="news-content">
        <small>{{ item.author }} · {{ item.publishDate }}</small>
        <p>{{ item.description }}</p>
        <a
          v-if="item.url?.startsWith('https://')"
          :href="item.url"
          target="_blank"
          rel="noopener noreferrer"
          >{{ text('Launch.Homepage.News.ViewDetail') }}</a
        >
      </div></PCard
    >
    <p v-if="!loading && !error && !items.length" class="pcl-empty">
      {{ text('Proto.Empty.News') }}
    </p>
  </section>
</template>
<style scoped>
.news-content {
  padding: 0 20px 20px;
}
.news-content p {
  white-space: pre-line;
  margin: 10px 0;
}
</style>
