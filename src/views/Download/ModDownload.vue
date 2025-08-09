<script setup lang="ts">
import modrinthApi, { type ISearchHit } from '@/api/modrinthApi'
import PCard from '@/components/widget/PCard.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import { onMounted, ref } from 'vue'

const hits = ref<ISearchHit[]>([])

const handleSearch = async (query?: string) => {
  hits.value.length = 0
  let data = await modrinthApi.searchProjects({ query, facets: [['project_type:mod']] })
  hits.value = data.hits
}

onMounted(() => {
  handleSearch()
})
</script>

<template>
  <PSearchBox @search="handleSearch" placeholder="搜索Mod" />
  <PCard hide-title v-if="hits && hits.length > 0">
    <PCompItem v-for="project in hits" v-bind="project" />
  </PCard>
  <div class="loading-page" v-else>
    <PLoading state="loading" />
  </div>
</template>
