<script setup lang="ts">
import modrinthApi, { type ISearchHit, type ProjectType } from '@/api/modrinthApi'
import PCard from '@/components/widget/PCard.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

const hits = ref<ISearchHit[]>([])
const route = useRoute()
const { t } = useI18n()

const handleSearch = async (query?: string) => {
  hits.value.length = 0
  let data = await modrinthApi.searchProjects({
    query,
    facets: [['project_type:' + route.meta.project_type]],
  })
  hits.value = data.hits
}

const placeholders: Record<ProjectType, string> = {
  mod: t('download.search_for') + ' ' + t('download.nav.mod'),
  modpack: t('download.search_for') + ' ' + t('download.nav.mod_pack'),
  resourcepack: t('download.search_for') + ' ' + t('download.nav.resource_pack'),
  shader: t('download.search_for') + ' ' + t('download.nav.shader_pack'),
}

watch(
  () => route.meta.project_type,
  (newType, oldType) => {
    if (newType !== oldType) {
      handleSearch()
    }
  },
  { immediate: true },
)
</script>

<template>
  <PSearchBox
    @search="handleSearch"
    :placeholder="placeholders[route.meta.project_type as ProjectType]"
  />
  <PCard hide-title v-if="hits && hits.length > 0">
    <PCompItem v-for="project in hits" :data="project" clickable />
  </PCard>
  <div class="loading-page" v-else>
    <PLoading state="loading" />
  </div>
</template>
