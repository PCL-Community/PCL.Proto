<script setup lang="ts">
import modrinthApi, {
  type ISearchHit,
  type ProjectType as ModrinthProjectType,
} from '@/api/modrinthApi'
import SetupItem from '@/components/widget/SetupItem.vue'
import PCard from '@/components/widget/PCard.vue'
import PCompItem from '@/components/widget/PCompItem.vue'
import PLoading from '@/components/widget/PLoading.vue'
import PSearchBox from '@/components/widget/PSearchBox.vue'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import IconRefresh from '@/components/icons/side/IconRefresh.vue'

const hits = ref<ISearchHit[]>([])
const route = useRoute()
const { t } = useI18n()
type ProjectType = ModrinthProjectType | 'world'

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
  world: t('download.search_for') + ' ' + t('download.nav.world'),
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
  <PCard hide-title>
    <div class="page-comp-options">
      <SetupItem label="版本" type="select" :options="[{ key: 'all', text: '全部' }]"></SetupItem>
      <SetupItem label="类型" type="select" :options="[{ key: 'all', text: '全部' }]"></SetupItem>
      <SetupItem label="来源" type="select" :options="[{ key: 'all', text: '全部' }]"></SetupItem>
    </div>
  </PCard>
  <PCard hide-title v-if="hits && hits.length > 0" ref="projectCard" v-card-drop-animate>
    <PCompItem v-for="project in hits" :data="project" clickable />
  </PCard>
  <div class="loading-page" v-else>
    <PLoading state="loading" />
  </div>
</template>

<style scoped lang="scss">
.page-comp-options {
  display: flex;
  gap: 1rem;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;

  :deep(.setupitem-with-text-container) {
    &:nth-child(1) {
      flex: 4;
    }
    &:nth-child(2) {
      flex: 3;
    }
    &:nth-child(3) {
      flex: 3;
    }
  }
}
</style>
